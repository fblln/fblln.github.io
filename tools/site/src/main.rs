//! Build-time portfolio renderer. Trunk first emits the document shell and the
//! hydrate-mode WASM bundle; this post-build step replaces only `<body>` with
//! Leptos SSR output whose hydration markers exactly match that bundle.

use std::{env, fs, path::PathBuf};

const BOOT: &str = "<main id=\"boot\" class=\"boot\" aria-labelledby=\"boot-title\"><span>FBLLN/WASM</span><h1 id=\"boot-title\">INITIALIZING RUST RUNTIME</h1><span class=\"boot-line\"></span></main>";
const NO_SCRIPT: &str = "<noscript><style>#boot{display:none}</style><p class=\"ssg-notice\">Interactive controls require JavaScript; all portfolio content remains available below.</p></noscript>";

fn main() {
    let out_root = env::args()
        .nth(1)
        .or_else(|| env::var("TRUNK_STAGING_DIR").ok())
        .map(PathBuf::from)
        .expect("pass an output dir or set TRUNK_STAGING_DIR");
    let index_path = out_root.join("index.html");
    let document = fs::read_to_string(&index_path).expect("read Trunk index");
    let app = fblln_portfolio::render_static_app();
    let rendered = inject_static_body(&document, &app).expect("index has one body element");
    fs::write(index_path, rendered).expect("write statically rendered index");
    println!("site: rendered portfolio HTML for hydration");
}

/// Replaces rather than appends the body so repeated watch builds are
/// idempotent. The app gets a dedicated hydration root, allowing boot and
/// no-script affordances to remain outside Leptos' DOM cursor.
fn inject_static_body(document: &str, app: &str) -> Option<String> {
    let open = document.find("<body>")? + "<body>".len();
    let close = document.rfind("</body>")?;
    (open <= close).then(|| {
        format!(
            "{}<body>{BOOT}{NO_SCRIPT}<div id=\"app\">{app}</div></body>{}",
            &document[..document.find("<body>").expect("body open")],
            &document[close + "</body>".len()..]
        )
    })
}

#[cfg(test)]
mod tests {
    use super::inject_static_body;

    /// Watch mode may reuse generated output, so injection must remove old
    /// placeholders and create exactly one stable hydration root every time.
    #[test]
    fn static_body_replacement_is_complete_and_unique() {
        let result = inject_static_body(
            "<!doctype html><html><head></head><body>old</body></html>",
            "<main>portfolio</main>",
        )
        .expect("body");

        assert!(!result.contains("old"));
        assert_eq!(result.matches("id=\"app\"").count(), 1);
        assert!(result.contains("<main>portfolio</main>"));
        assert!(result.contains("#boot{display:none}"));
    }

    #[test]
    fn documents_without_a_body_are_rejected() {
        assert!(inject_static_body("<html></html>", "app").is_none());
    }
}
