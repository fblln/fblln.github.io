use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{HtmlElement, HtmlInputElement, KeyboardEvent};

#[derive(Clone, Copy)]
struct Project {
    slug: &'static str,
    name: &'static str,
    category: &'static str,
    stack: &'static str,
    statement: &'static str,
    evidence: &'static str,
    detail: &'static str,
    metric: &'static str,
    metric_label: &'static str,
    image: &'static str,
    image_alt: &'static str,
    repo: &'static str,
    tags: &'static [&'static str],
}

const PROJECTS: [Project; 4] = [
    Project {
        slug: "race-telemetry",
        name: "Race Telemetry Workbench",
        category: "Telemetry",
        stack: ".NET · TIMESCALEDB · MCP · AI",
        statement: "Turn raw Formula 1 telemetry into engineering and race-strategy insight.",
        evidence: "A local-first analysis platform with replay, strategy, race control, head-to-head comparison, and an MCP-backed AI surface.",
        detail: "FastF1 data is imported into TimescaleDB and exposed through typed .NET query primitives. The same bounded contracts power an engineering desktop and autonomous analysis through MCP—keeping natural-language answers grounded in real session data.",
        metric: "42.7×",
        metric_label: "raw telemetry compression",
        image: "/assets/race-telemetry.webp",
        image_alt: "Race Telemetry Workbench replay interface",
        repo: "https://github.com/fblln/race-telemetry-workbench",
        tags: &["F1", "PostgreSQL", "OpenTelemetry", "Agents"],
    },
    Project {
        slug: "lithograph",
        name: "Lithograph",
        category: "Code Intelligence",
        stack: "RUST · TREE-SITTER · LADYBUGDB · MCP",
        statement: "Turn a source tree into a queryable, evidence-backed architecture graph.",
        evidence: "29 typed relations, 25 MCP tools, hybrid language resolution, drift detection, ADRs, and offline-first architecture documentation.",
        detail: "Lithograph treats repository understanding as a deterministic systems problem rather than a prompt. It combines syntax extraction, typed graph construction, search, architecture analysis, evidence tracking, incremental regeneration, and optional model augmentation.",
        metric: "29",
        metric_label: "typed relation kinds",
        image: "",
        image_alt: "",
        repo: "https://github.com/fblln/Lithograph",
        tags: &["Rust", "Graphs", "Local-first", "Code Intelligence"],
    },
    Project {
        slug: "ridgeline",
        name: "Ridgeline",
        category: "Geospatial",
        stack: "RUST · WASM · GDAL · THREE.JS",
        statement: "Transform a GPX route and elevation data into an interactive terrain artifact.",
        evidence: "A real asset pipeline for DEM sampling, projected geometry, terrain textures, route replay, and 7200×5400 export.",
        detail: "Ridgeline combines a high-performance geospatial pipeline with a browser-native viewer. Instead of hiding the work, it exposes each stage: GPX parsing, DEM acquisition, sampling, relief, slope, forest layers, and final asset handoff.",
        metric: "6.7×",
        metric_label: "Rust rastering speedup",
        image: "/assets/ridgeline.webp",
        image_alt: "Ridgeline 3D terrain visualization",
        repo: "https://github.com/fblln/ridgeline",
        tags: &["Rust", "WASM", "Terrain", "GPX"],
    },
    Project {
        slug: "apexline",
        name: "Apexline",
        category: "Research",
        stack: "PYTHON · FASTF1 · GEOMETRY · POLYLINES",
        statement: "Prove whether a telemetry lap has the same shape as an oracle circuit.",
        evidence: "26,689 race laps inspected across 24 circuits with auditable recovery, rejection, fitting, and compact polyline output.",
        detail: "Apexline normalizes lap-boundary overlap, rejects invalid evidence, fits closed paths without arbitrary warping, and reports residuals that explain whether each lap is useful, recoverable, suspicious, or invalid.",
        metric: "93.7%",
        metric_label: "2025 laps classified good",
        image: "/assets/apexline.svg",
        image_alt: "Apexline Canadian Grand Prix geometry diagnostics",
        repo: "https://github.com/fblln/apexline",
        tags: &["Geometry", "F1", "Validation", "Compression"],
    },
];

const CATEGORIES: [&str; 5] = [
    "All",
    "Telemetry",
    "Code Intelligence",
    "Geospatial",
    "Research",
];

// Fallback shown only if the browser exposes no Resource Timing entry for the
// WASM (very old browsers). Normal path reports the real bytes this client got.
const PRODUCTION_WASM_SIZE: &str = "155 KiB";

// The actual compressed bytes the browser downloaded for the WASM module, read
// from the Resource Timing API. `encoded_body_size` is the on-the-wire body
// (gzip on GitHub Pages); it stays non-zero on cache hits, unlike `transfer_size`.
fn wasm_transfer_size() -> Option<String> {
    let entries = web_sys::window()?
        .performance()?
        .get_entries_by_type("resource");
    entries
        .iter()
        .filter_map(|entry| entry.dyn_into::<web_sys::PerformanceResourceTiming>().ok())
        .find(|res| res.name().ends_with(".wasm"))
        .map(|res| res.encoded_body_size())
        .filter(|&bytes| bytes > 0.0)
        .map(human_bytes)
}

// Scales to KiB or MiB so the debug build (~3.7 MiB) doesn't read as "3800 KiB".
fn human_bytes(bytes: f64) -> String {
    if bytes >= 1024.0 * 1024.0 {
        format!("{:.1} MiB", bytes / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KiB", bytes / 1024.0)
    }
}

fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map(|performance| performance.now())
        .unwrap_or_default()
}

fn browser_engine() -> String {
    let agent = web_sys::window()
        .and_then(|window| window.navigator().user_agent().ok())
        .unwrap_or_else(|| "unknown".into());
    if agent.contains("Firefox") {
        "Gecko".into()
    } else if agent.contains("Chrome") || agent.contains("Chromium") {
        "Blink".into()
    } else if agent.contains("Safari") {
        "WebKit".into()
    } else {
        "Browser VM".into()
    }
}

fn focus_element(id: &str) {
    if let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
        .and_then(|element| element.dyn_into::<HtmlElement>().ok())
    {
        let _ = element.focus();
    }
}

fn project_matches(project: Project, category: &str, query: &str) -> bool {
    let category_match = category == "All" || project.category == category;
    let query = query.trim().to_ascii_lowercase();
    let query_match = query.is_empty()
        || project.name.to_ascii_lowercase().contains(&query)
        || project.statement.to_ascii_lowercase().contains(&query)
        || project.stack.to_ascii_lowercase().contains(&query)
        || project
            .tags
            .iter()
            .any(|tag| tag.to_ascii_lowercase().contains(&query));
    category_match && query_match
}

fn main() {
    console_error_panic_hook::set_once();
    // `performance.now()` here includes document loading, the Trunk loader, and
    // WebAssembly instantiation. It is intentionally captured before the UI mounts.
    let boot_time = now_ms();
    let wasm_size = wasm_transfer_size().unwrap_or_else(|| PRODUCTION_WASM_SIZE.to_string());
    if let Some(document) = web_sys::window().and_then(|window| window.document())
        && let Some(boot) = document.get_element_by_id("boot")
    {
        boot.remove();
    }
    leptos::mount::mount_to_body(move || {
        view! { <App boot_time=boot_time wasm_size=wasm_size.clone() /> }
    });
}

#[component]
fn App(boot_time: f64, wasm_size: String) -> impl IntoView {
    let category = RwSignal::new("All".to_string());
    let query = RwSignal::new(String::new());
    let active = RwSignal::new(0usize);
    let expanded = RwSignal::new(None::<usize>);
    let system_open = RwSignal::new(false);
    let boot_time = format!("{:.1} ms", boot_time);
    let hero_wasm_size = format!("{wasm_size} WASM");
    let engine = browser_engine();

    Effect::new(move |_| {
        if system_open.get() {
            focus_element("system-close");
        }
    });
    Effect::new(move |_| {
        if expanded.get().is_some() {
            focus_element("case-close");
        }
    });

    let key_handler = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
        let typing = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element())
            .map(|element| element.tag_name() == "INPUT")
            .unwrap_or(false);

        if event.key() == "Escape" {
            expanded.set(None);
            system_open.set(false);
            return;
        }
        if typing {
            return;
        }
        match event.key().as_str() {
            "/" => {
                event.prevent_default();
                if let Some(input) = web_sys::window()
                    .and_then(|window| window.document())
                    .and_then(|document| document.get_element_by_id("project-search"))
                    .and_then(|element| element.dyn_into::<HtmlInputElement>().ok())
                {
                    let _ = input.focus();
                }
            }
            "j" | "J" => active.update(|index| *index = (*index + 1) % PROJECTS.len()),
            "k" | "K" => {
                active.update(|index| *index = (*index + PROJECTS.len() - 1) % PROJECTS.len())
            }
            "Enter" => expanded.set(Some(active.get_untracked())),
            "s" | "S" => system_open.update(|open| *open = !*open),
            _ => {}
        }
    });
    if let Some(window) = web_sys::window() {
        let _ = window
            .add_event_listener_with_callback("keydown", key_handler.as_ref().unchecked_ref());
        key_handler.forget();
    }

    view! {
        <div class="site-shell">
            <a class="skip-link" href="#main-content">"Skip to main content"</a>
            <header class="topbar">
                <a class="wordmark" href="#top" aria-label="Fabio Ellena home">"FE/26"</a>
                <nav aria-label="Primary navigation">
                    <a href="#work">"Work"</a>
                    <a href="#impact">"Impact"</a>
                    <a href="#experience">"Experience"</a>
                    <a href="#contact">"Contact"</a>
                </nav>
                <button
                    class="runtime-button"
                    class:active=move || system_open.get()
                    aria-controls="system-panel"
                    aria-expanded=move || system_open.get().to_string()
                    on:click=move |_| system_open.update(|open| *open = !*open)
                >
                    <span class="status-dot"></span>"WASM/ACTIVE"
                </button>
            </header>

            <main id="main-content" tabindex="-1">
                <section id="top" class="hero section-grid">
                    <div class="section-index">"00"</div>
                    <div class="hero-content">
                        <p class="eyebrow">"FABIO ELLENA · SENIOR STAFF SOFTWARE ENGINEER · TURIN"</p>
                        <h1>"SOFTWARE SHOULD "<br/><em>"SURVIVE"</em><br/>"CONTACT WITH PRODUCTION."</h1>
                        <div class="runtime-facts" aria-label="Application runtime facts">
                            <span>"LEPTOS CSR"</span>
                            <span>"RUST/WASM"</span>
                            <span>"0 SERVER RENDERING"</span>
                            <span>{hero_wasm_size}</span>
                            <span>{format!("BOOT {}", boot_time)}</span>
                        </div>
                        <div class="hero-bottom">
                            <p>"I build distributed systems, telemetry platforms, and local-first AI tools for problems where scale, evidence, and reliability are not optional."</p>
                            <div class="hero-actions">
                                <a class="button button-solid" href="#work">"Inspect systems ↓"</a>
                                <a class="button" href="mailto:ellena.fabio@gmail.com">"Start a conversation ↗"</a>
                            </div>
                        </div>
                    </div>
                </section>

                <section id="impact" class="impact-grid" aria-label="Selected impact">
                    <div class="impact-cell"><strong>"10M+"</strong><span>"connected vehicles supported"</span></div>
                    <div class="impact-cell"><strong>"100+"</strong><span>"B2B and B2C APIs shaped"</span></div>
                    <div class="impact-cell"><strong>"0→20+"</strong><span>"engineering organization growth"</span></div>
                    <div class="impact-cell"><strong>"100K+"</strong><span>"IoT devices operated"</span></div>
                </section>

                <section id="work" class="work section-grid">
                    <div class="section-index">"01"</div>
                    <div>
                        <div class="section-heading">
                            <div><p class="eyebrow">"SELECTED SYSTEMS"</p><h2>"THE WORK"</h2></div>
                            <p>"Four public projects. Each one built around real data, explicit constraints, and an inspectable technical argument."</p>
                        </div>

                        <div class="project-controls">
                            <div class="category-list" aria-label="Filter projects">
                                {CATEGORIES.into_iter().map(|item| view! {
                                    <button
                                        class:active=move || category.get() == item
                                        on:click=move |_| category.set(item.to_string())
                                    >{item}</button>
                                }).collect_view()}
                            </div>
                            <label class="search-box" for="project-search">
                                <span>"/"</span>
                                <input
                                    id="project-search"
                                    type="search"
                                    placeholder="FILTER STACK OR CONCEPT"
                                    prop:value=move || query.get()
                                    on:input=move |event| query.set(event_target_value(&event))
                                />
                            </label>
                        </div>

                        <div class="project-list">
                            {PROJECTS.into_iter().enumerate().map(|(index, project)| view! {
                                <article
                                    class="project-row"
                                    class:active=move || active.get() == index
                                    class:hidden=move || !project_matches(project, &category.get(), &query.get())
                                    id=project.slug
                                >
                                    <button
                                        class="project-open"
                                        aria-controls="case-study"
                                        aria-expanded=move || (expanded.get() == Some(index)).to_string()
                                        aria-haspopup="dialog"
                                        on:click=move |_| { active.set(index); expanded.set(Some(index)); }
                                        aria-label=format!("Open {} case study", project.name)
                                    >
                                        <span class="project-number">{format!("{:02}", index + 1)}</span>
                                        <span class="project-name">{project.name}</span>
                                        <span class="project-stack">{project.stack}</span>
                                        <span class="project-arrow">"↗"</span>
                                    </button>
                                    <div class="project-summary">
                                        <p>{project.statement}</p>
                                        <div class="tag-list">{project.tags.iter().map(|tag| view! { <span>{*tag}</span> }).collect_view()}</div>
                                    </div>
                                </article>
                            }).collect_view()}
                        </div>
                        <p class="keyboard-hint">"KEYBOARD: J/K SELECT · ENTER INSPECT · / SEARCH · S SYSTEM · ESC CLOSE"</p>
                    </div>
                </section>

                <section class="principles section-grid">
                    <div class="section-index">"02"</div>
                    <div>
                        <div class="section-heading"><div><p class="eyebrow">"OPERATING PRINCIPLES"</p><h2>"HOW I BUILD"</h2></div></div>
                        <div class="principle-grid">
                            <article><span>"A"</span><h3>"MEASURE THE REAL SYSTEM"</h3><p>"Production distributions, traces, failure modes, and representative data beat architectural theatre."</p></article>
                            <article><span>"B"</span><h3>"MAKE EVIDENCE QUERYABLE"</h3><p>"Graphs, typed contracts, telemetry, and deterministic tests turn understanding into infrastructure."</p></article>
                            <article><span>"C"</span><h3>"SHIP THE SMALLEST SOLID THING"</h3><p>"A narrow production-grade loop creates more leverage than a broad demo that cannot be trusted."</p></article>
                        </div>
                    </div>
                </section>

                <section id="experience" class="experience section-grid">
                    <div class="section-index">"03"</div>
                    <div>
                        <div class="section-heading"><div><p class="eyebrow">"2017 → NOW"</p><h2>"EXPERIENCE"</h2></div><p>"From on-device ML to platforms serving millions of connected vehicles."</p></div>
                        <div class="timeline">
                            <article><time>"2024—NOW"</time><div><h3>"SENIOR STAFF SOFTWARE ENGINEER"</h3><p>"Stellantis · Connected Services"</p></div><p>"Leading the consolidation and technical strategy of global connected-service API platforms."</p></article>
                            <article><time>"2023—2024"</time><div><h3>"STAFF SOFTWARE ENGINEER"</h3><p>"Stellantis · Connected Services"</p></div><p>"Scaled engineering practices, developer experience, and architecture across a 20+ person organization."</p></article>
                            <article><time>"2022—2023"</time><div><h3>"TECHNICAL LEAD"</h3><p>"FCA · Connected Services"</p></div><p>"Led telemetry and ecosystem integrations for globally connected vehicle services."</p></article>
                            <article><time>"2018—2022"</time><div><h3>"SOFTWARE ENGINEER"</h3><p>"Concept Reply"</p></div><p>"Built cloud-native IoT, telemetry, and device platforms supporting 100K+ devices."</p></article>
                            <article><time>"2017—2018"</time><div><h3>"ML ENGINEER INTERN"</h3><p>"Docapost · Nice"</p></div><p>"Built constrained on-device document classification and reduced manual annotation by roughly 50%."</p></article>
                        </div>
                    </div>
                </section>

                <section class="education section-grid">
                    <div class="section-index">"04"</div>
                    <div class="education-layout">
                        <div><p class="eyebrow">"FOUNDATIONS"</p><h2>"TWO MASTER'S. ONE SYSTEMS MINDSET."</h2></div>
                        <div class="degree-list">
                            <article><strong>"M.SC. DATA SCIENCE"</strong><span>"EURECOM · TÉLÉCOM PARISTECH · GPA 4.0"</span></article>
                            <article><strong>"M.SC. COMPUTER ENGINEERING"</strong><span>"POLITECNICO DI TORINO · 110 CUM LAUDE"</span></article>
                            <article><strong>"B.SC. COMPUTER ENGINEERING"</strong><span>"POLITECNICO DI TORINO"</span></article>
                        </div>
                    </div>
                </section>

                <section id="contact" class="contact">
                    <img src="/assets/fabio.webp" alt="Fabio Ellena" width="460" height="460" />
                    <div>
                        <p class="eyebrow">"OPEN TO HARD PROBLEMS"</p>
                        <h2>"LET'S BUILD SOMETHING THAT HAS TO WORK."</h2>
                        <div class="contact-links">
                            <a href="mailto:ellena.fabio@gmail.com">"EMAIL ↗"</a>
                            <a href="https://www.linkedin.com/in/fabioellena/" target="_blank" rel="noreferrer">"LINKEDIN ↗"</a>
                            <a href="https://github.com/fblln" target="_blank" rel="noreferrer">"GITHUB ↗"</a>
                        </div>
                    </div>
                </section>
            </main>

            <footer>
                <span>"© 2026 FABIO ELLENA"</span>
                <span>"FULL RUST · LEPTOS · WASM32"</span>
                <button on:click=move |_| system_open.set(true)>"INSPECT RUNTIME [S]"</button>
            </footer>

            <aside id="system-panel" class="system-panel" class:open=move || system_open.get() hidden=move || !system_open.get() aria-label="Runtime diagnostics">
                <div class="panel-head"><span>"SYSTEM/DIAGNOSTICS"</span><button id="system-close" on:click=move |_| system_open.set(false)>"CLOSE [ESC]"</button></div>
                <div class="diagnostic-grid">
                    <div><span>"APPLICATION"</span><strong>"LEPTOS CSR"</strong></div>
                    <div><span>"TARGET"</span><strong>"WASM32-UNKNOWN-UNKNOWN"</strong></div>
                    <div><span>"BROWSER ENGINE"</span><strong>{engine}</strong></div>
                    <div><span>"BOOT TO WASM ENTRY"</span><strong>{boot_time}</strong></div>
                    <div><span>"WASM RECEIVED"</span><strong>{wasm_size}</strong></div>
                    <div><span>"RENDERER"</span><strong>"DOM + SVG"</strong></div>
                    <div><span>"BUNDLE BUDGET"</span><strong>"≤ 500 KIB GZIP"</strong></div>
                    <div><span>"APP CODE"</span><strong>"100% RUST"</strong></div>
                    <div><span>"BUILD"</span><strong>{option_env!("BUILD_SHA").unwrap_or("LOCAL")}</strong></div>
                </div>
                <p>"The browser received a Rust application compiled to WebAssembly. Project data is embedded at build time; there is no client-side GitHub API dependency."</p>
            </aside>

            <div id="case-study" class="case-overlay" class:open=move || expanded.get().is_some() hidden=move || expanded.get().is_none()>
                <button class="overlay-backdrop" aria-label="Close case study" on:click=move |_| expanded.set(None)></button>
                <article class="case-panel" role="dialog" aria-modal="true" aria-labelledby="case-title">
                    <div class="panel-head"><span>{move || expanded.get().map(|index| format!("CASE/{:02}", index + 1)).unwrap_or_default()}</span><button id="case-close" on:click=move |_| expanded.set(None)>"CLOSE [ESC]"</button></div>
                    <div class="case-body">
                        <p class="eyebrow">{move || expanded.get().map(|index| PROJECTS[index].stack).unwrap_or("")}</p>
                        <h2 id="case-title">{move || expanded.get().map(|index| PROJECTS[index].name).unwrap_or("")}</h2>
                        <p class="case-statement">{move || expanded.get().map(|index| PROJECTS[index].statement).unwrap_or("")}</p>
                        <div class="case-metric"><strong>{move || expanded.get().map(|index| PROJECTS[index].metric).unwrap_or("")}</strong><span>{move || expanded.get().map(|index| PROJECTS[index].metric_label).unwrap_or("")}</span></div>
                        <p>{move || expanded.get().map(|index| PROJECTS[index].detail).unwrap_or("")}</p>
                        <blockquote>{move || expanded.get().map(|index| PROJECTS[index].evidence).unwrap_or("")}</blockquote>
                        <div class="case-visual" class:graph=move || expanded.get() == Some(1)>
                            <img
                                class:hidden=move || expanded.get().map(|index| PROJECTS[index].image.is_empty()).unwrap_or(true)
                                src=move || expanded.get().map(|index| PROJECTS[index].image).unwrap_or("")
                                alt=move || expanded.get().map(|index| PROJECTS[index].image_alt).unwrap_or("")
                            />
                            <svg class="graph-visual" viewBox="0 0 720 300" role="img" aria-label="Semantic graph with connected code entities">
                                <g class="edges"><path d="M95 155L245 65L390 145L540 65L640 185"/><path d="M95 155L255 240L390 145L525 245L640 185"/><path d="M245 65L255 240M540 65L525 245"/></g>
                                <g class="nodes"><circle cx="95" cy="155" r="24"/><circle cx="245" cy="65" r="18"/><circle cx="255" cy="240" r="18"/><circle cx="390" cy="145" r="30"/><circle cx="540" cy="65" r="18"/><circle cx="525" cy="245" r="18"/><circle cx="640" cy="185" r="24"/></g>
                            </svg>
                        </div>
                        <a class="button button-solid" href=move || expanded.get().map(|index| PROJECTS[index].repo).unwrap_or("") target="_blank" rel="noreferrer">"INSPECT REPOSITORY ↗"</a>
                    </div>
                </article>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_slugs_are_unique() {
        for (index, project) in PROJECTS.iter().enumerate() {
            assert!(
                PROJECTS
                    .iter()
                    .skip(index + 1)
                    .all(|other| other.slug != project.slug)
            );
        }
    }

    #[test]
    fn every_project_has_evidence_and_a_repository() {
        for project in PROJECTS {
            assert!(!project.evidence.is_empty());
            assert!(project.repo.starts_with("https://github.com/fblln/"));
        }
    }

    #[test]
    fn filtering_covers_category_stack_and_tags() {
        assert!(project_matches(
            PROJECTS[1],
            "Code Intelligence",
            "tree-sitter"
        ));
        assert!(project_matches(PROJECTS[2], "All", "wasm"));
        assert!(!project_matches(PROJECTS[0], "Research", "telemetry"));
    }
}
