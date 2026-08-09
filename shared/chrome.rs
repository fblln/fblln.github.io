//! Canonical chrome markup shared by the interactive portfolio and the static
//! generators. The topbar and the runtime-diagnostics panel used to exist twice
//! — once as a Leptos `view!` and once as a `format!` string in the article
//! generator — which meant every chrome change had to be made in two languages
//! and verified on two surfaces. Emitting the same string on both sides removes
//! the class of bug where the markup drifts while the shared stylesheet keeps
//! insisting both surfaces look identical.
//!
//! Both builders take content the caller owns as `&'static`-shaped literals from
//! our own source, so they intentionally do no HTML escaping; anything derived
//! from user input must be escaped before it reaches here.

use crate::navigation::PRIMARY_NAV;

/// The wordmark, as an inline SVG data URI. Held here rather than in either
/// document because the portfolio shell and every generated page carry the same
/// mark, and a favicon that disagrees between routes reads as two sites.
///
/// `index.html` is copied verbatim by Trunk and never passes through Rust, so it
/// keeps its own `<link>`; the test below is what stops that copy from drifting.
pub const FAVICON_HREF: &str = "data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 64 64%22><rect width=%2264%22 height=%2264%22 fill=%22%230a0a0a%22/><path d=%22M14 12h36v8H23v9h22v8H23v15h-9z%22 fill=%22white%22/></svg>";

/// `base` prefixes home-relative fragments. The portfolio document passes `""`
/// because `#work` already targets itself; generated pages under another route
/// pass `"/"` so the same link doesn't scroll whichever document it landed in.
pub fn topbar(base: &str) -> String {
    let links: String = PRIMARY_NAV
        .iter()
        .map(|item| {
            let href = if item.href.starts_with('#') {
                format!("{base}{}", item.href)
            } else {
                item.href.to_string()
            };
            format!("<a href=\"{href}\">{}</a>", item.label)
        })
        .collect();
    // On the portfolio the wordmark is a same-document jump; everywhere else it
    // has to be a real navigation back to the root.
    let home = if base.is_empty() { "#top" } else { base };
    format!(
        "<header class=\"topbar\">\
<a class=\"wordmark\" href=\"{home}\" aria-label=\"Fabio Ellena home\">FE/26</a>\
<nav aria-label=\"Primary navigation\">{links}</nav>\
<button class=\"runtime-button\" type=\"button\" aria-controls=\"system-panel\" aria-expanded=\"false\"><span class=\"status-dot\"></span>WASM/ACTIVE</button>\
<details class=\"mobile-nav\"><summary aria-label=\"Toggle navigation menu\">MENU</summary>\
<nav aria-label=\"Primary navigation\">{links}</nav></details>\
</header>"
    )
}

/// Rows are `(id, label, value)`. A non-empty id puts a stable hook on the value
/// cell so the browser build can replace a build-time placeholder with a real
/// measurement without re-rendering the panel or knowing its layout.
pub fn system_panel(rows: &[(&str, &str, &str)], note: &str) -> String {
    let cells: String = rows
        .iter()
        .map(|(id, label, value)| {
            let attr = if id.is_empty() {
                String::new()
            } else {
                format!(" id=\"{id}\"")
            };
            format!("<div><span>{label}</span><strong{attr}>{value}</strong></div>")
        })
        .collect();
    // Ships hidden so the panel is inert before the bundle wires it up, rather
    // than flashing a full-height overlay on every cold load.
    format!(
        "<aside id=\"system-panel\" class=\"system-panel\" hidden aria-label=\"Runtime diagnostics\">\
<div class=\"panel-head\"><span>SYSTEM/DIAGNOSTICS</span><button id=\"system-close\" type=\"button\">CLOSE [ESC]</button></div>\
<div class=\"diagnostic-grid\">{cells}</div>\
<p>{note}</p>\
</aside>"
    )
}

#[cfg(test)]
mod tests {
    use super::{FAVICON_HREF, system_panel, topbar};

    /// The portfolio shell is hand-authored HTML that Rust never rewrites, so
    /// its favicon is a copy of this constant. Nothing but this test notices
    /// when the mark changes here and not there.
    #[test]
    fn the_portfolio_shell_carries_the_shared_favicon() {
        assert!(
            include_str!("../index.html").contains(FAVICON_HREF),
            "index.html's favicon has drifted from chrome::FAVICON_HREF"
        );
    }

    /// Static pages live below `/articles/`, so leaving fragment-only links in
    /// either navigation variant would scroll the wrong document — while the
    /// portfolio needs those exact fragments left alone.
    #[test]
    fn topbar_rebases_fragments_only_for_pages_below_the_root() {
        let generated = topbar("/");
        for href in ["/#work", "/#capabilities", "/#experience", "/#contact"] {
            assert_eq!(generated.matches(&format!("href=\"{href}\"")).count(), 2);
        }
        assert_eq!(generated.matches("href=\"/articles/\"").count(), 2);
        assert!(!generated.contains("href=\"#"));
        assert!(generated.contains("<a class=\"wordmark\" href=\"/\""));

        let portfolio = topbar("");
        assert_eq!(portfolio.matches("href=\"#work\"").count(), 2);
        assert!(portfolio.contains("<a class=\"wordmark\" href=\"#top\""));
        assert!(!portfolio.contains("href=\"/#"));
    }

    /// Both surfaces render the same button and panel target, because the
    /// stylesheet and the enhancer address them by exactly these names.
    #[test]
    fn chrome_keeps_the_hooks_both_surfaces_address() {
        for base in ["", "/"] {
            let html = topbar(base);
            assert!(html.contains("class=\"runtime-button\""));
            assert!(html.contains("aria-controls=\"system-panel\""));
            assert!(html.contains("aria-expanded=\"false\""));
        }
        let panel = system_panel(&[("", "TARGET", "WASM32")], "note");
        assert!(panel.contains("id=\"system-panel\""));
        assert!(panel.contains("id=\"system-close\""));
        assert!(panel.contains(" hidden "));
    }

    /// An id on a row is the contract the browser build uses to overwrite a
    /// deterministic placeholder; a row without one must stay a plain cell.
    #[test]
    fn panel_rows_expose_ids_only_where_requested() {
        let html = system_panel(
            &[
                ("fact-engine", "BROWSER ENGINE", "Browser VM"),
                ("", "APP CODE", "100% RUST"),
            ],
            "why",
        );

        assert!(html.contains("<strong id=\"fact-engine\">Browser VM</strong>"));
        assert!(html.contains("<span>APP CODE</span><strong>100% RUST</strong>"));
        assert_eq!(html.matches("<strong").count(), 2);
        assert!(html.contains("<p>why</p>"));
    }
}
