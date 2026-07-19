//! Browser entry point. The portfolio view lives in the library so the same
//! component tree can be rendered statically on the host and hydrated in WASM.

#[cfg(target_arch = "wasm32")]
fn main() {
    fblln_portfolio::run();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
