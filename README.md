# fblln.github.io

Fabio Ellena's technical portfolio: statically generated with Leptos, hydrated
by Rust/WebAssembly for interaction, and deployed to GitHub Pages.

## Stack

- Rust 2024
- Leptos SSG + hydration
- Trunk
- `wasm-bindgen` / `web-sys`
- GitHub Actions and GitHub Pages

The application has no handwritten JavaScript. A native build of the same
Leptos view writes the portfolio HTML; Trunk emits the loader that hydrates it.
Writing is generated from Markdown and progressively enhanced by the same WASM.

## Local development

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
trunk serve --open
```

## Validation

```bash
cargo fmt --check
cargo test
cargo test --no-default-features --features ssr
cargo test --manifest-path tools/site/Cargo.toml
cargo test --manifest-path tools/blog/Cargo.toml
cargo check --target wasm32-unknown-unknown --no-default-features --features hydrate
trunk build --release
```

The CI build compresses the final WASM bundle and enforces a 500 KiB gzip budget.
