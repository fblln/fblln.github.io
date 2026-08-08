//! Interactive instruments embedded in article pages.
//!
//! An instrument exists for one reason: a concept a reader reliably gets wrong
//! from a static diagram. The bar is deliberately high — most figures are better
//! as the inline SVG the articles already use, because a diagram costs an hour
//! and an instrument costs weeks. Three of the seven articles have one; the
//! others were assessed and refused, which is the bar doing its job.
//!
//! | Instrument | Article | What a diagram cannot show |
//! |---|---|---|
//! | `dtls-cid` | The Lookup That Saves the Handshake | The cost of anchoring on an address, and that CID alone is not enough |
//! | `sram` | Two Kilobytes and No Operating System | A failure that raises nothing at all |
//! | `isolated` | Performance Is Making Gradle Less Gradle | That the speedup is a restriction, not an optimisation |
//!
//! Each instrument keeps its simulation in a `logic` submodule that is pure and
//! compiles on every target, so `cargo test` covers the argument natively and the
//! browser only ever renders it. The shared vocabulary below is deliberately
//! thin — it emerged from three real cases rather than being designed up front,
//! and it is only the log line, because that is genuinely all three have in
//! common. The panel chrome is shared as CSS, not as Rust.

pub(crate) mod dtls_cid;
pub(crate) mod isolated;
pub(crate) mod sram;

/// Severity of a log line, so a view can mark outcomes without re-deriving what
/// happened. `Held` is the interesting one: a mechanism deliberately refusing to
/// act is neither success nor failure, and every instrument here has that case.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Level {
    Ok,
    Held,
    Fail,
}

impl Level {
    /// The CSS class the log renders with. Kept beside the enum so a new level
    /// cannot be added without deciding how it looks.
    pub(crate) fn class(self) -> &'static str {
        match self {
            Level::Ok => "ok",
            Level::Held => "held",
            Level::Fail => "fail",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Line {
    pub(crate) level: Level,
    pub(crate) text: String,
}

/// A bounded run log. Instruments are embedded in pages a reader may leave open
/// for a long time, so an unbounded transcript is a slow leak; the panel only
/// ever shows a short tail anyway.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Log {
    lines: Vec<Line>,
}

impl Log {
    const CAP: usize = 24;
    const TAIL: usize = 6;

    pub(crate) fn note(&mut self, level: Level, text: String) {
        self.lines.push(Line { level, text });
        if self.lines.len() > Self::CAP {
            self.lines.remove(0);
        }
    }

    /// Test-only accessors: the views read `tail()` and nothing else, so these
    /// exist purely to let the log's bounding be asserted.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.lines.len()
    }

    /// The visible tail, oldest first.
    pub(crate) fn tail(&self) -> &[Line] {
        let n = self.lines.len();
        &self.lines[n.saturating_sub(Self::TAIL)..]
    }

    #[cfg(test)]
    pub(crate) fn last(&self) -> Option<&Line> {
        self.lines.last()
    }
}

/// Mounts every instrument placeholder the page declares. A page with none —
/// which is most articles — pays only this one query.
#[cfg(target_arch = "wasm32")]
pub(crate) fn mount_all(doc: &web_sys::Document) {
    use wasm_bindgen::JsCast;

    let Ok(nodes) = doc.query_selector_all("[data-instrument]") else {
        return;
    };
    for i in 0..nodes.length() {
        let Some(node) = nodes.item(i) else { continue };
        let host: web_sys::HtmlElement = node.unchecked_into();
        let name = host.get_attribute("data-instrument");
        // The placeholder ships a static summary so the article still makes its
        // point without the bundle. `mount_to` appends, so clear it first or the
        // reader sees both.
        match name.as_deref() {
            Some("dtls-cid") => {
                host.set_inner_html("");
                leptos::mount::mount_to(host, || {
                    leptos::view! { <dtls_cid::view::DtlsCid /> }
                })
                .forget();
            }
            Some("sram") => {
                host.set_inner_html("");
                leptos::mount::mount_to(host, || leptos::view! { <sram::view::Sram /> }).forget();
            }
            Some("isolated") => {
                host.set_inner_html("");
                leptos::mount::mount_to(host, || {
                    leptos::view! { <isolated::view::Isolated /> }
                })
                .forget();
            }
            // An unknown name leaves the static fallback in place rather than
            // blanking the figure — the article still reads without any of this.
            _ => continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_carry_their_own_class() {
        assert_eq!(Level::Ok.class(), "ok");
        assert_eq!(Level::Held.class(), "held");
        assert_eq!(Level::Fail.class(), "fail");
    }

    #[test]
    fn a_fresh_log_is_empty() {
        let log = Log::default();
        assert_eq!(log.len(), 0);
        assert_eq!(log.tail().len(), 0);
        assert!(log.last().is_none());
    }

    #[test]
    fn the_tail_is_short_until_six_lines_exist() {
        let mut log = Log::default();
        for i in 1..=4 {
            log.note(Level::Ok, format!("line {i}"));
            assert_eq!(log.tail().len(), i);
        }
    }

    #[test]
    fn the_log_is_bounded_and_the_tail_is_the_last_six() {
        let mut log = Log::default();
        for i in 0..80 {
            log.note(Level::Ok, format!("line {i}"));
        }
        assert_eq!(log.len(), Log::CAP, "log must not grow without limit");
        assert_eq!(log.tail().len(), Log::TAIL);
        assert_eq!(log.tail().last(), log.last());
        assert_eq!(log.last().map(|l| l.text.as_str()), Some("line 79"));
        // The oldest surviving line is CAP back from the newest, not line 0.
        assert_eq!(log.tail().first().map(|l| l.text.as_str()), Some("line 74"));
    }
}
