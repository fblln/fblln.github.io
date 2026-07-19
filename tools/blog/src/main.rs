//! Static article generator. Reads `content/articles/*.md` (TOML front matter +
//! Markdown) and writes SEO-friendly static HTML under `<staging>/articles/`,
//! plus an index, per-tag pages, an Atom feed, and a refreshed sitemap.xml.
//!
//! Run by trunk's post_build hook; the output dir comes from TRUNK_STAGING_DIR
//! (an explicit arg overrides it for manual runs). Code blocks are highlighted
//! at build time with syntect — no client-side JS.

#[path = "../../../shared/navigation.rs"]
mod navigation;

use std::{env, fs, path::PathBuf, sync::OnceLock};

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use serde::Deserialize;
use syntect::highlighting::ThemeSet;
use syntect::html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

const BASE: &str = "https://fblln.github.io";
const AUTHOR: &str = "Fabio Ellena";
const READING_CSS: &str = include_str!("../article.css");
const SHARED_TYPOGRAPHY_CSS: &str = include_str!("../../../shared/typography.css");
const SHARED_HEADER_CSS: &str = include_str!("../../../shared/header.css");

/// `<script>` that loads the site's wasm bundle to enhance article pages (copy
/// buttons, reading-progress bar). Empty when no bundle is present — e.g. a manual
/// generator run outside a Trunk build — so pages just stay static.
static ENHANCE_SCRIPT: OnceLock<String> = OnceLock::new();

#[derive(Deserialize)]
struct FrontMatter {
    title: String,
    date: String, // ISO "YYYY-MM-DD"
    #[serde(default)]
    description: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    draft: bool,
}

struct Head {
    level: u8,
    text: String,
    id: String,
}

struct Article {
    slug: String,
    fm: FrontMatter,
    body: String,
    toc: Vec<Head>,
    minutes: usize,
}

impl Article {
    fn url(&self) -> String {
        format!("{BASE}/articles/{}/", self.slug)
    }
}

fn main() {
    let out_root = env::args()
        .nth(1)
        .or_else(|| env::var("TRUNK_STAGING_DIR").ok())
        .map(PathBuf::from)
        .expect("pass an output dir or set TRUNK_STAGING_DIR");
    let articles_dir = out_root.join("articles");
    // The staging directory is reused during `trunk serve`. Recreate this
    // generator-owned subtree so moving a source into `drafts/` removes its old
    // route, feed entry, and tag page instead of leaving stale public files.
    if articles_dir.exists() {
        fs::remove_dir_all(&articles_dir).expect("clear generated articles dir");
    }
    fs::create_dir_all(&articles_dir).expect("create articles dir");

    // Trunk emits the wasm loader as `<name>-<hash>.js` at the dist root; find it so
    // article pages can pull the same bundle in to enhance themselves.
    // wasm-bindgen's loader defaults to the *unhashed* `<crate>_bg.wasm`; Trunk
    // renames the wasm with a content hash, so pass the real path to init().
    let script = find_bundle(&out_root)
        .map(|js| {
            let wasm = js.replace(".js", "_bg.wasm");
            format!("<script type=\"module\">import init from \"/{js}\";init({{module_or_path:\"/{wasm}\"}})</script>")
        })
        .unwrap_or_default();
    let _ = ENHANCE_SCRIPT.set(script);

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let code_css =
        css_for_theme_with_class_style(&ts.themes["base16-ocean.dark"], ClassStyle::Spaced)
            .expect("code theme css");
    fs::write(articles_dir.join("article.css"), article_css(&code_css)).expect("write article.css");

    let mut articles: Vec<Article> = Vec::new();
    let content = PathBuf::from("content/articles");
    if let Ok(entries) = fs::read_dir(&content) {
        for entry in entries {
            let path = entry.expect("dir entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let raw = fs::read_to_string(&path).expect("read md");
            let stem = path.file_stem().unwrap().to_string_lossy();
            let slug = strip_date_prefix(&stem);
            if let Some(article) = parse(&slug, &raw, &ss) {
                articles.push(article);
            }
        }
    }
    // Newest first.
    articles.sort_by(|a, b| b.fm.date.cmp(&a.fm.date));

    for (i, a) in articles.iter().enumerate() {
        let dir = articles_dir.join(&a.slug);
        fs::create_dir_all(&dir).expect("create article dir");
        // Articles are newest-first, so the previous index is the newer post.
        let newer = i.checked_sub(1).map(|j| &articles[j]);
        let older = articles.get(i + 1);
        fs::write(dir.join("index.html"), article_page(a, newer, older)).expect("write article");
    }

    fs::write(
        articles_dir.join("index.html"),
        index_page("Notes & essays", None, &articles),
    )
    .expect("write index");

    // Per-tag pages.
    let mut tags: Vec<String> = articles.iter().flat_map(|a| a.fm.tags.clone()).collect();
    tags.sort();
    tags.dedup();
    for tag in &tags {
        let dir = articles_dir.join("tags").join(slug(tag));
        fs::create_dir_all(&dir).expect("create tag dir");
        let subset: Vec<&Article> = articles
            .iter()
            .filter(|a| a.fm.tags.contains(tag))
            .collect();
        fs::write(
            dir.join("index.html"),
            index_page(
                &format!("Tagged “{tag}”"),
                Some(tag),
                &subset_owned(&subset),
            ),
        )
        .expect("write tag page");
    }

    fs::write(articles_dir.join("feed.xml"), atom_feed(&articles)).expect("write feed");
    fs::write(out_root.join("sitemap.xml"), sitemap(&articles)).expect("write sitemap");

    println!(
        "blog: generated {} article(s), {} tag(s)",
        articles.len(),
        tags.len()
    );
}

/// Locate Trunk's wasm loader JS at the dist root (`index-<hash>.js`, i.e. a
/// top-level `.js` that isn't the `_bg` wasm shim), returning its file name.
fn find_bundle(dist: &std::path::Path) -> Option<String> {
    fs::read_dir(dist)
        .ok()?
        .filter_map(|e| e.ok())
        .find_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            (name.ends_with(".js") && !name.ends_with("_bg.js")).then_some(name)
        })
}

/// Split TOML front matter (`+++ ... +++`) from the Markdown body, render it.
fn parse(slug: &str, raw: &str, ss: &SyntaxSet) -> Option<Article> {
    let rest = raw.strip_prefix("+++")?;
    let end = rest.find("+++")?;
    let fm: FrontMatter = toml::from_str(rest[..end].trim()).expect("front matter");
    if fm.draft {
        return None;
    }
    let md = rest[end + 3..].trim_start();
    let words = md.split_whitespace().count();
    let minutes = (words / 200).max(1);
    let (body, toc) = render(md, ss);
    Some(Article {
        slug: slug.to_string(),
        fm,
        body,
        toc,
        minutes,
    })
}

/// Render Markdown to HTML, highlighting code blocks and giving headings ids +
/// a table of contents (h2/h3).
fn render(md: &str, ss: &SyntaxSet) -> (String, Vec<Head>) {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);

    let mut events: Vec<Event> = Vec::new();
    let mut toc: Vec<Head> = Vec::new();
    let mut code: Option<(String, String)> = None;
    let mut heading: Option<(u8, String)> = None;

    for event in Parser::new_ext(md, opts) {
        if let Some((_, buf)) = heading.as_mut() {
            match &event {
                Event::Text(t) | Event::Code(t) => buf.push_str(t),
                Event::End(TagEnd::Heading(_)) => {
                    let (level, text) = heading.take().unwrap();
                    let id = slug(&text);
                    if (2..=3).contains(&level) {
                        toc.push(Head {
                            level,
                            text: text.clone(),
                            id: id.clone(),
                        });
                    }
                    events.push(Event::Html(
                        format!(
                            "<h{level} id=\"{id}\"><a class=\"anchor\" href=\"#{id}\" aria-hidden=\"true\">#</a>{}</h{level}>",
                            esc(&text)
                        )
                        .into(),
                    ));
                }
                _ => {}
            }
            continue;
        }
        if let Some((_, buf)) = code.as_mut() {
            match &event {
                Event::Text(t) => buf.push_str(t),
                Event::End(TagEnd::CodeBlock) => {
                    let (lang, src) = code.take().unwrap();
                    events.push(Event::Html(highlight(&lang, &src, ss).into()));
                }
                _ => {}
            }
            continue;
        }
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading = Some((hlevel(level), String::new()))
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(l) => l.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                code = Some((lang, String::new()));
            }
            // Wrap tables so wide ones scroll on narrow screens instead of cramping.
            Event::Start(Tag::Table(a)) => {
                events.push(Event::Html("<div class=\"table-wrap\">".into()));
                events.push(Event::Start(Tag::Table(a)));
            }
            Event::End(TagEnd::Table) => {
                events.push(Event::End(TagEnd::Table));
                events.push(Event::Html("</div>".into()));
            }
            ev => events.push(ev),
        }
    }

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    (html, toc)
}

fn highlight(lang: &str, source: &str, ss: &SyntaxSet) -> String {
    let syntax = ss
        .find_syntax_by_token(lang)
        .or_else(|| ss.find_syntax_by_extension(lang))
        .unwrap_or_else(|| ss.find_syntax_plain_text());
    let mut gen = ClassedHTMLGenerator::new_with_class_style(syntax, ss, ClassStyle::Spaced);
    for line in LinesWithEndings::from(source) {
        // ponytail: skip a line that fails to tokenize rather than abort the build.
        let _ = gen.parse_html_for_line_which_includes_newline(line);
    }
    let code = gen.finalize();
    if lang.is_empty() {
        format!("<pre class=\"code\"><code>{code}</code></pre>")
    } else {
        // Fenced blocks with a language get a header bar showing the language.
        let l = esc(lang);
        format!("<figure class=\"codeblock\"><figcaption>{l}</figcaption><pre class=\"code\" data-lang=\"{l}\"><code>{code}</code></pre></figure>")
    }
}

fn hlevel(h: HeadingLevel) -> u8 {
    match h {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

// ---- templates ----

/// Renders the static header from the same navigation data used by Leptos. Home
/// section fragments gain a root prefix because article pages live below
/// `/articles/`; Writing stays absolute. The runtime button is enhanced by the
/// shared WASM bundle when available and remains harmless without JavaScript.
fn topbar() -> String {
    let links: String = navigation::PRIMARY_NAV
        .iter()
        .map(|item| {
            let href = if item.href.starts_with('#') {
                format!("/{}", item.href)
            } else {
                item.href.to_string()
            };
            format!("<a href=\"{href}\">{}</a>", item.label)
        })
        .collect();
    format!("<header class=\"topbar\">\
<a class=\"wordmark\" href=\"/\" aria-label=\"Fabio Ellena home\">FE/26</a>\
<nav aria-label=\"Primary navigation\">{links}</nav>\
<button class=\"runtime-button\" type=\"button\" aria-controls=\"system-panel\" aria-expanded=\"false\"><span class=\"status-dot\"></span>WASM/ACTIVE</button>\
<details class=\"mobile-nav\"><summary aria-label=\"Toggle navigation menu\">MENU</summary>\
<nav aria-label=\"Primary navigation\">{links}</nav></details>\
</header>")
}

/// Composes Writing-specific rules around the shared type and header contracts.
/// The order matters: article layout may specialize base tokens, then the
/// shared header reasserts its chrome without duplicating either source file.
fn article_css(code_css: &str) -> String {
    format!("{SHARED_TYPOGRAPHY_CSS}\n{READING_CSS}\n{SHARED_HEADER_CSS}\n{code_css}\n")
}

const FOOTER: &str =
    "<footer class=\"site\"><span>© 2026 Fabio Ellena</span><span><a href=\"/articles/feed.xml\">RSS</a></span></footer>";

fn shell(head: &str, body: &str) -> String {
    let enhance = ENHANCE_SCRIPT.get().map(String::as_str).unwrap_or("");
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<meta name=\"theme-color\" content=\"#f2f0e9\">\
<link rel=\"icon\" href=\"data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 64 64%22><rect width=%2264%22 height=%2264%22 fill=%22%230a0a0a%22/><path d=%22M14 12h36v8H23v9h22v8H23v15h-9z%22 fill=%22white%22/></svg>\">{head}\
<link rel=\"stylesheet\" href=\"/articles/article.css\">\
<link rel=\"alternate\" type=\"application/atom+xml\" href=\"/articles/feed.xml\" title=\"Fabio Ellena — Writing\">\
</head><body>{topbar}{body}{FOOTER}{enhance}</body></html>", topbar = topbar()
    )
}

fn article_page(a: &Article, newer: Option<&Article>, older: Option<&Article>) -> String {
    let url = a.url();
    let desc = esc(&a.fm.description);
    let title = esc(&a.fm.title);
    let head = format!(
        "<title>{title} — Fabio Ellena</title>\
<meta name=\"description\" content=\"{desc}\">\
<link rel=\"canonical\" href=\"{url}\">\
<meta property=\"og:type\" content=\"article\">\
<meta property=\"og:title\" content=\"{title}\">\
<meta property=\"og:description\" content=\"{desc}\">\
<meta property=\"og:url\" content=\"{url}\">\
<meta name=\"twitter:card\" content=\"summary\">"
    );
    let tags: String =
        a.fm.tags
            .iter()
            .map(|t| {
                format!(
                    "<a class=\"tag\" href=\"/articles/tags/{}/\">{}</a>",
                    slug(t),
                    esc(t)
                )
            })
            .collect();
    let body = format!(
        "<main><p class=\"eyebrow\">{date} · {min} min read</p>\
<h1 class=\"title\">{title}</h1>\
<div class=\"post-meta\">{tags}</div>{toc}<article>{body}</article>{nav}</main>",
        date = esc(&a.fm.date),
        min = a.minutes,
        toc = toc_html(&a.toc),
        body = a.body,
        nav = post_nav(newer, older),
    );
    shell(&head, &body)
}

/// Bottom-of-article links to the chronologically adjacent posts.
fn post_nav(newer: Option<&Article>, older: Option<&Article>) -> String {
    if newer.is_none() && older.is_none() {
        return String::new();
    }
    let link = |a: &Article, class: &str, label: &str| {
        format!(
            "<a class=\"{class}\" href=\"/articles/{slug}/\"><span class=\"pn-dir\">{label}</span>\
<span class=\"pn-title\">{title}</span></a>",
            slug = a.slug,
            title = esc(&a.fm.title),
        )
    };
    format!(
        "<nav class=\"post-nav\">{}{}</nav>",
        older
            .map(|o| link(o, "older", "← Older"))
            .unwrap_or_default(),
        newer
            .map(|n| link(n, "newer", "Newer →"))
            .unwrap_or_default(),
    )
}

fn toc_html(heads: &[Head]) -> String {
    if heads.is_empty() {
        return String::new();
    }
    let items: String = heads
        .iter()
        .map(|h| {
            format!(
                "<li class=\"lvl{}\"><a href=\"#{}\">{}</a></li>",
                h.level,
                h.id,
                esc(&h.text)
            )
        })
        .collect();
    format!("<nav class=\"toc\"><p class=\"toc-title\">Contents</p><ul>{items}</ul></nav>")
}

fn index_page(title: &str, tag: Option<&str>, articles: &[Article]) -> String {
    let etitle = esc(title);
    let canonical = match tag {
        Some(t) => format!("{BASE}/articles/tags/{}/", slug(t)),
        None => format!("{BASE}/articles/"),
    };
    let head = format!(
        "<title>{etitle} — Fabio Ellena</title>\
<meta name=\"description\" content=\"Technical writing by Fabio Ellena.\">\
<link rel=\"canonical\" href=\"{canonical}\">"
    );
    let items: String = articles
        .iter()
        .map(|a| {
            let tags: String = a
                .fm
                .tags
                .iter()
                .map(|t| format!("<a class=\"tag\" href=\"/articles/tags/{}/\">{}</a>", slug(t), esc(t)))
                .collect();
            format!(
                "<li class=\"post-item\"><a class=\"post-title\" href=\"/articles/{slug}/\">{title}</a>\
<p>{desc}</p><div class=\"post-meta\"><span>{date}</span><span>{min} min</span>{tags}</div></li>",
                slug = a.slug,
                title = esc(&a.fm.title),
                desc = esc(&a.fm.description),
                date = esc(&a.fm.date),
                min = a.minutes,
            )
        })
        .collect();
    let list = if articles.is_empty() {
        "<p class=\"eyebrow\">Nothing published yet.</p>".to_string()
    } else {
        format!("<ul class=\"post-list\">{items}</ul>")
    };
    let body = format!(
        "<main><p class=\"eyebrow\">Writing</p><h1 class=\"title\">{etitle}</h1>{list}</main>"
    );
    shell(&head, &body)
}

// index_page takes &[Article]; tag subsets are &[&Article], so clone-project.
fn subset_owned(subset: &[&Article]) -> Vec<Article> {
    subset
        .iter()
        .map(|a| Article {
            slug: a.slug.clone(),
            fm: FrontMatter {
                title: a.fm.title.clone(),
                date: a.fm.date.clone(),
                description: a.fm.description.clone(),
                tags: a.fm.tags.clone(),
                draft: false,
            },
            body: String::new(),
            toc: Vec::new(),
            minutes: a.minutes,
        })
        .collect()
}

fn atom_feed(articles: &[Article]) -> String {
    let updated = articles
        .first()
        .map(|a| a.fm.date.as_str())
        .unwrap_or("1970-01-01");
    let entries: String = articles
        .iter()
        .map(|a| {
            format!(
                "<entry><title>{title}</title><link href=\"{url}\"/><id>{url}</id>\
<updated>{date}T00:00:00Z</updated><summary>{desc}</summary></entry>",
                title = esc(&a.fm.title),
                url = a.url(),
                date = esc(&a.fm.date),
                desc = esc(&a.fm.description),
            )
        })
        .collect();
    format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
<feed xmlns=\"http://www.w3.org/2005/Atom\">\
<title>Fabio Ellena — Writing</title>\
<link href=\"{BASE}/articles/feed.xml\" rel=\"self\"/>\
<link href=\"{BASE}/articles/\"/>\
<id>{BASE}/articles/</id>\
<updated>{updated}T00:00:00Z</updated>\
<author><name>{AUTHOR}</name></author>{entries}</feed>\n"
    )
}

fn sitemap(articles: &[Article]) -> String {
    let mut urls = format!("<url><loc>{BASE}/</loc></url><url><loc>{BASE}/articles/</loc></url>");
    for a in articles {
        urls.push_str(&format!("<url><loc>{}</loc></url>", a.url()));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">{urls}</urlset>\n"
    )
}

// ---- small helpers ----

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Drop a leading `YYYY-MM-DD-` from a filename so `2026-07-17-foo.md` becomes
/// the URL slug `foo`.
fn strip_date_prefix(stem: &str) -> String {
    let b = stem.as_bytes();
    let dated = b.len() > 11
        && b[10] == b'-'
        && b[..10].iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        });
    if dated {
        stem[11..].to_string()
    } else {
        stem.to_string()
    }
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::{article_css, topbar, SHARED_HEADER_CSS, SHARED_TYPOGRAPHY_CSS};

    /// Static pages live below `/articles/`, so leaving fragment-only links in
    /// either desktop or mobile navigation would scroll the wrong document.
    #[test]
    fn topbar_rebases_home_fragments_in_both_navigation_variants() {
        let html = topbar();

        for href in ["/#work", "/#capabilities", "/#experience", "/#contact"] {
            assert_eq!(html.matches(&format!("href=\"{href}\"")).count(), 2);
        }
        assert_eq!(html.matches("href=\"/articles/\"").count(), 2);
        assert!(!html.contains("href=\"#"));
        assert!(html.contains("aria-controls=\"system-panel\""));
    }

    /// Fonts must be declared before article layout consumes the variables, and
    /// the header must follow that layout so its border/type contract wins over
    /// reading-surface specializations.
    #[test]
    fn article_styles_preserve_shared_contract_order() {
        let css = article_css("/* syntax */");
        let typography = css.find(SHARED_TYPOGRAPHY_CSS).expect("typography css");
        let header = css.find(SHARED_HEADER_CSS).expect("header css");
        let syntax = css.find("/* syntax */").expect("syntax css");

        assert_eq!(typography, 0);
        assert!(typography < header);
        assert!(header < syntax);
    }
}
