//! Build-time portfolio renderer. Trunk first emits the document shell and the
//! hydrate-mode WASM bundle; this post-build step replaces only `<body>` with
//! Leptos SSR output whose hydration markers exactly match that bundle.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

const BOOT: &str = "<main id=\"boot\" class=\"boot\" aria-labelledby=\"boot-title\"><span>FBLLN/WASM</span><h1 id=\"boot-title\">INITIALIZING RUST RUNTIME</h1><span class=\"boot-line\"></span></main>";
const NO_SCRIPT: &str = "<noscript><style>#boot{display:none}</style><p class=\"ssg-notice\">Interactive controls require JavaScript; all portfolio content remains available below.</p></noscript>";
const CRITICAL_FONT_PRELOADS: &str = concat!(
    "<link rel=\"preload\" href=\"/assets/fonts/archivo-400.woff2\" as=\"font\" type=\"font/woff2\" crossorigin>",
    "<link rel=\"preload\" href=\"/assets/fonts/archivo-700.woff2\" as=\"font\" type=\"font/woff2\" crossorigin>",
    "<link rel=\"preload\" href=\"/assets/fonts/archivo-800.woff2\" as=\"font\" type=\"font/woff2\" crossorigin>",
    "<link rel=\"preload\" href=\"/assets/fonts/archivo-900.woff2\" as=\"font\" type=\"font/woff2\" crossorigin>",
    "<link rel=\"preload\" href=\"/assets/fonts/ibm-plex-mono-500.woff2?v=2\" as=\"font\" type=\"font/woff2\" crossorigin>",
    "<link rel=\"preload\" href=\"/assets/fonts/ibm-plex-mono-700.woff2?v=2\" as=\"font\" type=\"font/woff2\" crossorigin>"
);

fn main() {
    let out_root = env::args()
        .nth(1)
        .or_else(|| env::var("TRUNK_STAGING_DIR").ok())
        .map(PathBuf::from)
        .expect("pass an output dir or set TRUNK_STAGING_DIR");
    let index_path = out_root.join("index.html");
    let document = fs::read_to_string(&index_path).expect("read Trunk index");
    let app = fblln_portfolio::render_static_app();
    // Trunk emits both this preload and wasm-bindgen's later `fetch()`. Their
    // request modes differ, so browsers cannot reuse the response and download
    // the module twice. Keep the JS module preload, but remove only the WASM
    // fetch preload before writing the final document.
    let document = remove_redundant_wasm_preload(&document);
    let rendered = inject_static_body(&document, &app).expect("index has one body element");
    let rendered = inline_critical_styles(&rendered, &out_root);
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

/// Removes Trunk's WASM `as=fetch` preload while preserving every other link.
///
/// wasm-bindgen owns module instantiation and fetches the hashed WASM URL
/// itself. Trunk's separate anonymous preload is not cache-compatible with
/// that fetch on GitHub Pages, causing a second transfer instead of a warm
/// hand-off. Attribute order is deliberately ignored because Trunk controls
/// the generated markup and may reorder attributes between releases.
fn remove_redundant_wasm_preload(document: &str) -> String {
    let mut result = String::with_capacity(document.len());
    let mut remainder = document;

    while let Some(start) = remainder.find("<link") {
        result.push_str(&remainder[..start]);
        let tag_end = match remainder[start..].find('>') {
            Some(end) => start + end + 1,
            None => {
                result.push_str(&remainder[start..]);
                return result;
            }
        };
        let tag = &remainder[start..tag_end];
        let is_wasm_fetch_preload = tag.contains("rel=\"preload\"")
            && tag.contains("as=\"fetch\"")
            && tag.contains(".wasm");

        if !is_wasm_fetch_preload {
            result.push_str(tag);
        }
        remainder = &remainder[tag_end..];
    }

    result.push_str(remainder);
    result
}

/// Inlines the portfolio's three small, render-critical stylesheets after the
/// post-build minifier has processed them. GitHub Pages cannot provide HTTP/2
/// Early Hints or connection-level tuning, so on cold navigations three tiny
/// CSS requests cost more in round trips than they save in document bytes.
/// Keeping the authored files lets Trunk continue to hash and watch them; only
/// the final portfolio document changes its delivery form.
fn inline_critical_styles(document: &str, out_root: &Path) -> String {
    let mut result = String::with_capacity(document.len());
    let mut remainder = document;
    let mut added_font_preloads = false;

    while let Some(start) = remainder.find("<link") {
        result.push_str(&remainder[..start]);
        let tag_end = match remainder[start..].find('>') {
            Some(end) => start + end + 1,
            None => {
                result.push_str(&remainder[start..]);
                return result;
            }
        };
        let tag = &remainder[start..tag_end];

        if let Some(path) = critical_style_path(tag) {
            if let Ok(css) = fs::read_to_string(out_root.join(path)) {
                if !added_font_preloads {
                    // All six faces are used by the statically rendered first
                    // viewport. Starting them at document discovery removes
                    // the CSS→font request chain and avoids fallback reflow.
                    result.push_str(CRITICAL_FONT_PRELOADS);
                    added_font_preloads = true;
                }
                result.push_str("<style>");
                result.push_str(&css);
                result.push_str("</style>");
            } else {
                // A partially-written watch build must stay renderable. Keep
                // Trunk's original link until its hashed asset is available.
                result.push_str(tag);
            }
        } else {
            result.push_str(tag);
        }
        remainder = &remainder[tag_end..];
    }

    result.push_str(remainder);
    result
}

/// Returns the generated local filename only for the portfolio CSS bundle.
/// The prefix check deliberately excludes Writing's article stylesheet and any
/// future third-party CSS, whose loading strategy may have different needs.
fn critical_style_path(tag: &str) -> Option<&str> {
    let href_start = tag.find("href=\"")? + "href=\"".len();
    let href_end = href_start + tag[href_start..].find('"')?;
    let path = tag[href_start..href_end].strip_prefix('/')?;
    let is_stylesheet = tag.contains("rel=\"stylesheet\"");
    let is_critical = ["styles-", "typography-", "header-"]
        .iter()
        .any(|prefix| path.starts_with(prefix) && path.ends_with(".css"));
    (is_stylesheet && is_critical).then_some(path)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        critical_style_path, inject_static_body, inline_critical_styles, remove_redundant_wasm_preload,
    };

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

    /// The loader retains its JS module preload, but the duplicate resource
    /// fetch must be gone regardless of Trunk's link-attribute ordering.
    #[test]
    fn removes_only_wasm_fetch_preload() {
        let result = remove_redundant_wasm_preload(
            "<head><link rel=\"modulepreload\" href=\"/app.js\"><link href=\"/app_bg.wasm\" as=\"fetch\" rel=\"preload\" crossorigin=\"anonymous\"><link rel=\"stylesheet\" href=\"/site.css\"></head>",
        );

        assert!(!result.contains("app_bg.wasm"));
        assert!(result.contains("modulepreload"));
        assert!(result.contains("stylesheet"));
    }

    #[test]
    fn keeps_non_wasm_fetch_preloads() {
        let source = "<link rel=\"preload\" href=\"/font.woff2\" as=\"fetch\">";
        assert_eq!(remove_redundant_wasm_preload(source), source);
    }

    #[test]
    fn identifies_only_the_hashed_portfolio_stylesheets() {
        assert_eq!(
            critical_style_path("<link rel=\"stylesheet\" href=\"/styles-a1.css\">"),
            Some("styles-a1.css")
        );
        assert_eq!(
            critical_style_path("<link rel=\"stylesheet\" href=\"/articles/article.css\">"),
            None
        );
        assert_eq!(
            critical_style_path("<link rel=\"preload\" href=\"/header-a1.css\">"),
            None
        );
        assert_eq!(critical_style_path("<link rel=\"stylesheet\">"), None);
    }

    #[test]
    fn inlines_available_critical_css_and_preserves_everything_else() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("fblln-site-{nonce}"));
        fs::create_dir(&directory).expect("temp dir");
        fs::write(directory.join("styles-a1.css"), "body{color:red}").expect("css");

        let source = "<head><link rel=\"stylesheet\" href=\"/styles-a1.css\"><link rel=\"stylesheet\" href=\"/articles/article.css\"><link rel=\"stylesheet\" href=\"/header-missing.css\"></head>";
        let result = inline_critical_styles(source, &directory);

        assert!(result.contains("<style>body{color:red}</style>"));
        assert_eq!(result.matches("rel=\"preload\"").count(), 6);
        assert!(result.contains("archivo-900.woff2"));
        assert!(result.contains("ibm-plex-mono-700.woff2?v=2"));
        assert!(result.contains("href=\"/articles/article.css\""));
        assert!(result.contains("href=\"/header-missing.css\""));
        fs::remove_dir_all(directory).expect("remove temp dir");
    }

    #[test]
    fn malformed_link_suffix_is_preserved() {
        let source = "<head><link rel=\"stylesheet\" href=\"/styles-a1.css";
        assert_eq!(inline_critical_styles(source, Path::new("/missing")), source);
    }
}
