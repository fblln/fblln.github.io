//! Browser runtime diagnostics that have deterministic server-rendered defaults.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlElement;

pub(crate) const PRODUCTION_WASM_SIZE: &str = "80 KiB";

/// Reads the compressed response size rather than transfer size so cached WASM
/// still reports the module size the user received on its original fetch.
#[cfg(target_arch = "wasm32")]
pub(crate) fn wasm_transfer_size() -> Option<String> {
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

#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn human_bytes(bytes: f64) -> String {
    if bytes >= 1024.0 * 1024.0 {
        format!("{:.1} MiB", bytes / (1024.0 * 1024.0))
    } else {
        format!("{:.0} KiB", bytes / 1024.0)
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn now_ms() -> f64 {
    web_sys::window()
        .and_then(|window| window.performance())
        .map(|performance| performance.now())
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn browser_engine() -> String {
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

#[cfg(target_arch = "wasm32")]
pub(crate) fn focus_element(id: &str) {
    if let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
        .and_then(|element| element.dyn_into::<HtmlElement>().ok())
    {
        let _ = element.focus();
    }
}

#[cfg(test)]
mod tests {
    use super::human_bytes;
    #[test]
    fn byte_sizes_cross_the_binary_unit_boundary() {
        assert_eq!(human_bytes(512.0 * 1024.0), "512 KiB");
        assert_eq!(human_bytes(1.5 * 1024.0 * 1024.0), "1.5 MiB");
    }
}
