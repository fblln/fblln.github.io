//! `isolated` — instruments "Performance Is Making Gradle Less Gradle".
//!
//! Readers reliably take Isolated Projects for an optimisation: the same builds,
//! executed faster. The article's argument is that it is nothing of the kind.
//! Configuration can run in parallel and cache per project only because whole
//! categories of previously legal build logic have been made **illegal**. The
//! speed is not earned by better scheduling; it is bought by removing capability.
//!
//! That is a hard thing to draw, because the interesting outcome is a build
//! *failure* on code that used to work. So the instrument gives the reader both
//! levers. Add a cross-project reference — the sort of thing a four-hundred-module
//! repository is full of — and then try to turn isolation on:
//!
//! - **Not isolated.** Any project may reach into any other, so no project's
//!   configuration can be trusted independently of the rest. Every project
//!   reconfigures, serially, on every build. Nothing can be cached per project.
//! - **Isolated.** Only changed projects reconfigure, in parallel across workers.
//!   But the cross-project reference is now a build failure, not a slow path.
//!
//! The wall-clock figures are an arithmetic model, not a measurement, and both
//! the instrument and its caption say so — a fabricated benchmark would be worse
//! than no number at all.

pub(crate) mod logic {
    use crate::instrument::{Level, Log};

    /// A mid-sized multi-project build. Small enough to see, large enough that
    /// serial configuration is visibly the wrong shape.
    pub(crate) const PROJECTS: usize = 8;

    /// Modelled configuration time for one project.
    pub(crate) const CONFIG_MS: u32 = 120;

    /// Parallel workers available for project configuration.
    pub(crate) const WORKERS: usize = 4;

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub(crate) struct Build {
        pub(crate) isolated: bool,
        /// Cross-project references the reader has added, as (from, to) indices.
        pub(crate) cross_refs: Vec<(usize, usize)>,
        /// Projects edited since the last successful configure.
        pub(crate) dirty: [bool; PROJECTS],
        /// Cross-project accesses rejected by isolation.
        pub(crate) rejected: usize,
        /// Wall-clock of the last configure, or None if it failed / never ran.
        pub(crate) last_ms: Option<u32>,
        /// Projects served from the configuration cache on the last run.
        pub(crate) cached: usize,
        pub(crate) failed: bool,
        pub(crate) log: Log,
    }

    impl Default for Build {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Build {
        pub(crate) fn new() -> Self {
            let mut build = Self {
                isolated: false,
                cross_refs: Vec::new(),
                // A cold build has nothing cached, so every project is dirty.
                dirty: [true; PROJECTS],
                rejected: 0,
                last_ms: None,
                cached: 0,
                failed: false,
                log: Log::default(),
            };
            build.log.note(
                Level::Ok,
                format!(
                    "cold build · {PROJECTS} projects · cross-project access allowed · {CONFIG_MS} ms to configure each (model)"
                ),
            );
            build
        }

        pub(crate) fn dirty_count(&self) -> usize {
            self.dirty.iter().filter(|d| **d).count()
        }

        /// Whether project `i` will reconfigure on the next `configure()`.
        ///
        /// This is not the same as "is dirty", and the difference is the whole
        /// point: without isolation, no project's configuration can be trusted
        /// independently of the rest, so *every* project reconfigures whatever
        /// changed. The grid therefore stays fully lit in that mode, which is
        /// what the article is arguing. Reporting a clean project as "cached"
        /// there would claim a saving that does not exist.
        pub(crate) fn will_reconfigure(&self, i: usize) -> bool {
            !self.isolated || self.dirty.get(i).copied().unwrap_or(false)
        }

        /// Toggling isolation does not itself fail — it is a mode change. The
        /// failure surfaces on the next configure, which is exactly how the real
        /// migration feels: you turn the flag on and then find out what your
        /// build was relying on.
        pub(crate) fn set_isolated(&mut self, isolated: bool) {
            if self.isolated == isolated {
                return;
            }
            self.isolated = isolated;
            self.last_ms = None;
            self.failed = false;
            // The cache is keyed on the model; changing the model invalidates it.
            self.dirty = [true; PROJECTS];
            self.log.note(
                Level::Ok,
                if isolated {
                    "isolated projects ON · per-project configuration caching now possible · cross-project access now illegal".to_string()
                } else {
                    "isolated projects OFF · any project may mutate any other · configuration must be serial".to_string()
                },
            );
        }

        /// Add the kind of reference the article describes: one project reaching
        /// into another to read or mutate its configuration.
        pub(crate) fn add_cross_ref(&mut self) {
            let from = self.cross_refs.len() % PROJECTS;
            let to = (from + 3) % PROJECTS;
            self.cross_refs.push((from, to));
            self.dirty = [true; PROJECTS];
            self.failed = false;
            self.last_ms = None;
            self.log.note(
                Level::Held,
                format!(
                    "project-{from} now reads project-{to}'s configuration · {} cross-project reference(s)",
                    self.cross_refs.len()
                ),
            );
        }

        pub(crate) fn edit_project(&mut self) {
            // Edit whichever project is currently clean, else the first.
            let idx = self.dirty.iter().position(|d| !*d).unwrap_or(0);
            self.dirty[idx] = true;
            self.failed = false;
            self.log.note(
                Level::Ok,
                format!(
                    "edited project-{idx} · {} project(s) dirty",
                    self.dirty_count()
                ),
            );
        }

        /// Run configuration. Returns the modelled wall-clock, or None if the
        /// build failed.
        pub(crate) fn configure(&mut self) -> Option<u32> {
            if self.isolated && !self.cross_refs.is_empty() {
                self.rejected += self.cross_refs.len();
                self.failed = true;
                self.last_ms = None;
                let (from, to) = self.cross_refs[0];
                self.log.note(
                    Level::Fail,
                    format!(
                        "BUILD FAILED · project-{from} cannot access project-{to} under isolation · {} violation(s) — this build configured fine before the flag",
                        self.cross_refs.len()
                    ),
                );
                return None;
            }

            self.failed = false;
            let ms = if self.isolated {
                // Only changed projects reconfigure, spread across the workers.
                let n = self.dirty_count();
                self.cached = PROJECTS - n;
                let waves = n.div_ceil(WORKERS) as u32;
                let ms = waves * CONFIG_MS;
                self.log.note(
                    Level::Ok,
                    format!(
                        "configured {n} project(s) across {WORKERS} workers · {} served from cache · {ms} ms",
                        self.cached
                    ),
                );
                ms
            } else {
                // No project's configuration can be trusted independently, so
                // every project reconfigures, in order, every time.
                self.cached = 0;
                let ms = PROJECTS as u32 * CONFIG_MS;
                self.log.note(
                    Level::Held,
                    format!(
                        "configured all {PROJECTS} projects serially · nothing cacheable per project · {ms} ms"
                    ),
                );
                ms
            };
            self.dirty = [false; PROJECTS];
            self.last_ms = Some(ms);
            Some(ms)
        }

        pub(crate) fn reset(&mut self) {
            *self = Self::new();
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn a_cold_build_has_everything_dirty_and_nothing_measured() {
            let build = Build::new();
            assert!(!build.isolated);
            assert_eq!(build.dirty_count(), PROJECTS);
            assert_eq!(build.last_ms, None);
            assert_eq!(build.rejected, 0);
            assert_eq!(build, Build::default());
        }

        #[test]
        fn without_isolation_every_project_reconfigures_serially() {
            let mut build = Build::new();
            assert_eq!(build.configure(), Some(PROJECTS as u32 * CONFIG_MS));
            assert_eq!(build.cached, 0);
            // Even with a single edit, all of them run again.
            build.edit_project();
            assert_eq!(build.configure(), Some(PROJECTS as u32 * CONFIG_MS));
            assert_eq!(build.cached, 0, "nothing is cacheable per project");
        }

        #[test]
        fn isolation_configures_only_dirty_projects_in_parallel() {
            let mut build = Build::new();
            build.set_isolated(true);
            // Cold: all 8 dirty, 4 workers → 2 waves.
            assert_eq!(build.configure(), Some(2 * CONFIG_MS));
            build.edit_project();
            assert_eq!(build.dirty_count(), 1);
            // One dirty project → one wave, seven from cache.
            assert_eq!(build.configure(), Some(CONFIG_MS));
            assert_eq!(build.cached, PROJECTS - 1);
        }

        #[test]
        fn toggling_to_the_same_mode_changes_nothing() {
            let mut build = Build::new();
            build.configure();
            let before = build.clone();
            build.set_isolated(false);
            assert_eq!(build, before);
        }

        #[test]
        fn switching_modes_invalidates_the_cache() {
            let mut build = Build::new();
            build.set_isolated(true);
            build.configure();
            assert_eq!(build.dirty_count(), 0);
            build.set_isolated(false);
            assert_eq!(build.dirty_count(), PROJECTS);
            assert_eq!(build.last_ms, None);
        }

        #[test]
        fn editing_marks_a_clean_project_dirty_and_never_overflows() {
            let mut build = Build::new();
            build.set_isolated(true);
            build.configure();
            for expected in 1..=PROJECTS {
                build.edit_project();
                assert_eq!(build.dirty_count(), expected);
            }
            // All dirty now — editing again must stay in range.
            build.edit_project();
            assert_eq!(build.dirty_count(), PROJECTS);
        }

        /// The article's thesis, pinned: the cross-project reference that a
        /// non-isolated build configures happily is a hard failure once isolation
        /// is on. Any mutation that downgrades this to a slow path fails here.
        #[test]
        fn invariant_a_cross_project_reference_configures_until_isolation_forbids_it() {
            let mut build = Build::new();
            build.add_cross_ref();
            assert_eq!(
                build.configure(),
                Some(PROJECTS as u32 * CONFIG_MS),
                "the reference is legal without isolation"
            );
            assert!(!build.failed);

            build.set_isolated(true);
            assert_eq!(build.configure(), None, "the same build must now fail");
            assert!(build.failed);
            assert_eq!(build.rejected, 1);
            assert_eq!(build.last_ms, None, "a failed build has no wall-clock");
        }

        /// Isolation is only ever faster, never slower — and the speedup is
        /// bounded by the worker count, not by cleverness.
        #[test]
        fn invariant_isolated_configuration_is_bounded_by_the_worker_count() {
            for edits in 0..=PROJECTS {
                let mut build = Build::new();
                build.set_isolated(true);
                build.configure();
                for _ in 0..edits {
                    build.edit_project();
                }
                let ms = build.configure().expect("no cross refs, must succeed");
                let n = edits.max(0);
                let expected = (n.div_ceil(WORKERS) as u32) * CONFIG_MS;
                assert_eq!(ms, expected, "for {edits} edits");
                assert!(
                    ms <= PROJECTS as u32 * CONFIG_MS,
                    "isolation must never be slower than serial"
                );
            }
        }

        /// A build with no cross-project references is never rejected, however
        /// many times it is configured in either mode.
        #[test]
        fn invariant_clean_builds_are_never_rejected() {
            let mut build = Build::new();
            for round in 0..12 {
                build.set_isolated(round % 2 == 0);
                build.edit_project();
                assert!(build.configure().is_some(), "round {round}");
                assert!(!build.failed);
            }
            assert_eq!(build.rejected, 0);
        }

        /// Without isolation a "clean" project is not a saved project — it will
        /// reconfigure anyway. Claiming otherwise in the grid would advertise a
        /// cache that does not exist.
        #[test]
        fn without_isolation_every_project_always_reconfigures_next_time() {
            let mut build = Build::new();
            build.configure();
            assert_eq!(build.dirty_count(), 0, "all clean after a configure");
            for i in 0..PROJECTS {
                assert!(
                    build.will_reconfigure(i),
                    "project-{i} is clean but must still reconfigure without isolation"
                );
            }
        }

        #[test]
        fn under_isolation_only_dirty_projects_reconfigure_next_time() {
            let mut build = Build::new();
            build.set_isolated(true);
            build.configure();
            for i in 0..PROJECTS {
                assert!(!build.will_reconfigure(i), "project-{i} should be cached");
            }
            build.edit_project();
            let hot = (0..PROJECTS).filter(|i| build.will_reconfigure(*i)).count();
            assert_eq!(hot, 1, "exactly the edited project reconfigures");
        }

        #[test]
        fn will_reconfigure_is_out_of_range_safe() {
            let mut build = Build::new();
            build.set_isolated(true);
            build.configure();
            assert!(!build.will_reconfigure(PROJECTS + 5));
        }

        #[test]
        fn reset_restores_the_cold_build() {
            let mut build = Build::new();
            build.add_cross_ref();
            build.set_isolated(true);
            build.configure();
            build.reset();
            assert_eq!(build, Build::new());
            assert!(build.cross_refs.is_empty());
            assert_eq!(build.rejected, 0);
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) mod view {
    use super::logic::{Build, CONFIG_MS, PROJECTS, WORKERS};
    use leptos::prelude::*;

    #[component]
    pub(crate) fn Isolated() -> impl IntoView {
        let b = RwSignal::new(Build::new());

        view! {
            <div class="inst" role="group" aria-label="Gradle isolated projects instrument">
                <div class="inst-head">
                    <span>"BUILD/CONFIGURATION"</span>
                    <span class="inst-anchor">
                        {move || if b.with(|b| b.isolated) { "ISOLATED PROJECTS: ON" } else { "ISOLATED PROJECTS: OFF" }}
                    </span>
                </div>

                <div class="inst-controls">
                    <button
                        type="button"
                        class:active=move || b.with(|b| !b.isolated)
                        on:click=move |_| b.update(|b| b.set_isolated(false))
                    >"CROSS-PROJECT ACCESS ALLOWED"</button>
                    <button
                        type="button"
                        class:active=move || b.with(|b| b.isolated)
                        on:click=move |_| b.update(|b| b.set_isolated(true))
                    >"ISOLATED"</button>
                </div>

                <div class="inst-grid" role="img"
                    aria-label=move || b.with(|b| format!(
                        "{} of {PROJECTS} projects need configuring", b.dirty_count()))>
                    {move || b.with(|b| (0..PROJECTS).map(|i| {
                        let involved = b.cross_refs.iter().any(|(f, t)| *f == i || *t == i);
                        let class = if b.failed && involved {
                            "cell bad"
                        } else if b.will_reconfigure(i) {
                            "cell dirty"
                        } else {
                            "cell cached"
                        };
                        view! { <span class=class>{format!("p{i}")}</span> }
                    }).collect_view())}
                </div>
                <div class="inst-legend">
                    <span><i class="cell dirty"></i>"NEEDS CONFIGURING"</span>
                    <span><i class="cell cached"></i>"FROM CACHE"</span>
                    <span><i class="cell bad"></i>"ILLEGAL ACCESS"</span>
                </div>

                <div class="inst-readout">
                    <div>
                        <span>"PROJECTS DIRTY"</span>
                        <strong>{move || b.with(|b| format!("{} / {PROJECTS}", b.dirty_count()))}</strong>
                    </div>
                    <div>
                        <span>"SERVED FROM CACHE"</span>
                        <strong>{move || b.with(|b| b.cached)}</strong>
                    </div>
                    <div>
                        <span>"CROSS-PROJECT REFS"</span>
                        <strong>{move || b.with(|b| b.cross_refs.len())}</strong>
                    </div>
                    <div>
                        <span>"REJECTED"</span>
                        <strong>{move || b.with(|b| b.rejected)}</strong>
                    </div>
                    <div class="inst-cost">
                        <span>"CONFIGURATION (MODEL)"</span>
                        <strong class:hot=move || b.with(|b| b.failed)>
                            {move || b.with(|b| match b.last_ms {
                                Some(ms) => format!("{ms} ms"),
                                None if b.failed => "FAILED".to_string(),
                                None => "—".to_string(),
                            })}
                        </strong>
                    </div>
                </div>

                <Show when=move || b.with(|b| b.failed)>
                    <p class="inst-pending">
                        "This build configured successfully a moment ago. Nothing about it got
                         slower — a category of build logic it was using simply stopped being legal.
                         That is what bought the parallelism."
                    </p>
                </Show>

                <div class="inst-actions">
                    <button type="button" on:click=move |_| b.update(|b| { b.configure(); })>
                        "CONFIGURE"
                    </button>
                    <button type="button" on:click=move |_| b.update(|b| b.edit_project())>
                        "EDIT ONE PROJECT"
                    </button>
                    <button type="button" on:click=move |_| b.update(|b| b.add_cross_ref())>
                        "ADD CROSS-PROJECT REFERENCE"
                    </button>
                    <button type="button" on:click=move |_| b.update(|b| b.reset())>"RESET"</button>
                </div>

                <p class="inst-note">
                    {format!("Model, not a measurement: {PROJECTS} projects at {CONFIG_MS} ms each, {WORKERS} workers. \
                              Serial configuration reconfigures all of them because any project may have mutated any other.")}
                </p>

                <ol class="inst-log">
                    {move || b.with(|b| b.log.tail().iter().map(|line| {
                        view! { <li class=line.level.class()>{line.text.clone()}</li> }
                    }).collect_view())}
                </ol>
            </div>
        }
    }
}
