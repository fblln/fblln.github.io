# fblln.github.io

Fabio Ellena's technical portfolio: a full client-side Rust application compiled to WebAssembly and deployed to GitHub Pages.

## Stack

- Rust 2024
- Leptos CSR
- Trunk
- `wasm-bindgen` / `web-sys`
- GitHub Actions and GitHub Pages

The application has no handwritten JavaScript. Trunk emits the minimal loader required to instantiate the WebAssembly module.

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
cargo check --target wasm32-unknown-unknown
trunk build --release
```

The CI build compresses the final WASM bundle and enforces a 500 KiB gzip budget.

