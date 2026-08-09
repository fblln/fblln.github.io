//! The contract between the two surfaces of this site: the interactive
//! portfolio, compiled to WebAssembly, and the static pages the generators
//! write. Both render the same header, resolve the same navigation, and load
//! the same design tokens, without either one being able to call the other.
//!
//! This was six loose files pulled in with `#[path]` for a long time, and the
//! files were never the problem — there was only ever one copy of each. What
//! made it a crate is that `chrome` needs `navigation`, and under `#[path]` that
//! private dependency surfaced in every consumer: `tools/site` had to declare a
//! `navigation` module it never once referenced, purely so `chrome` could find
//! it. A file included by path gets to dictate its host's module tree, and
//! nothing reports that but a confusing unresolved-module error.
//!
//! Inside a crate the dependency is internal, callers name only what they use,
//! and the stylesheets are read once here instead of by a `../../../` path in
//! every crate that wants them.

pub mod chrome;
pub mod instruments;
pub mod navigation;

/// The design tokens. One definition of every colour, hairline, and spacing
/// step, because two surfaces once disagreed about `--line` while sharing the
/// stylesheet that consumed it.
pub const TOKENS_CSS: &str = include_str!("tokens.css");

/// The type contract: `@font-face` sources and the fallback stacks, kept
/// together so navigating between surfaces cannot change the fallback glyphs.
pub const TYPOGRAPHY_CSS: &str = include_str!("typography.css");

/// The header, styled from the tokens above and rendered by [`chrome::topbar`].
pub const HEADER_CSS: &str = include_str!("header.css");

#[cfg(test)]
mod tests {
    use super::{HEADER_CSS, TOKENS_CSS, TYPOGRAPHY_CSS};

    /// The stylesheets are exported as strings so callers can assert against
    /// them; an empty one would make every such assertion vacuously pass.
    #[test]
    fn the_exported_stylesheets_are_not_empty() {
        for (name, css) in [
            ("tokens", TOKENS_CSS),
            ("typography", TYPOGRAPHY_CSS),
            ("header", HEADER_CSS),
        ] {
            assert!(!css.trim().is_empty(), "{name}.css is empty");
        }
    }

    /// `header.css` styles chrome that both surfaces render, so it must reach
    /// for tokens by name rather than carrying its own literals.
    #[test]
    fn only_the_token_sheet_declares_tokens() {
        assert!(TOKENS_CSS.contains("--ink:"));
        assert!(!HEADER_CSS.contains("--ink:"));
        assert!(!TYPOGRAPHY_CSS.contains("--ink:"));
    }
}
