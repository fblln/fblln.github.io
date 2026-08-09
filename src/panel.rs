//! Runtime-diagnostics panel behaviour, for both surfaces.
//!
//! The markup now ships statically on the portfolio and on every article page
//! (see `shared/chrome.rs`), so this module only attaches behaviour to markup
//! that already exists. That replaces two implementations of one panel: a
//! reactive Leptos `<aside>` driven by a signal, and a near-identical copy that
//! the article enhancer used to build with `innerHTML`.
//!
//! State lives in the DOM rather than in a signal because the panel sits outside
//! the hydration root — the portfolio's keyboard model and footer button reach
//! it through these functions instead of through the reactive graph.

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{Document, Element, KeyboardEvent};

fn document() -> Option<Document> {
    web_sys::window().and_then(|window| window.document())
}

fn parts(doc: &Document) -> Option<(Element, Element)> {
    let panel = doc.query_selector("#system-panel").ok().flatten()?;
    let trigger = doc.query_selector(".runtime-button").ok().flatten()?;
    Some((panel, trigger))
}

fn is_open(panel: &Element) -> bool {
    !panel.has_attribute("hidden")
}

fn set_open(open: bool) {
    let Some(doc) = document() else { return };
    let Some((panel, trigger)) = parts(&doc) else {
        return;
    };
    if open {
        let _ = panel.remove_attribute("hidden");
        let _ = panel.set_attribute("class", "system-panel open");
        let _ = trigger.set_attribute("class", "runtime-button active");
        let _ = trigger.set_attribute("aria-expanded", "true");
        // Moving focus into the panel is what makes Escape a predictable exit
        // for keyboard users rather than a shortcut they have to know about.
        crate::runtime::focus_element("system-close");
    } else {
        let _ = panel.set_attribute("hidden", "");
        let _ = panel.set_attribute("class", "system-panel");
        let _ = trigger.set_attribute("class", "runtime-button");
        let _ = trigger.set_attribute("aria-expanded", "false");
    }
}

pub fn open() {
    set_open(true);
}

pub fn close() {
    set_open(false);
}

pub fn toggle() {
    let Some(doc) = document() else { return };
    let Some((panel, _)) = parts(&doc) else {
        return;
    };
    set_open(!is_open(&panel));
}

/// Attaches the trigger, the close button, and Escape. Safe to call on a page
/// that carries no panel — every lookup fails closed.
pub fn wire() {
    let Some(doc) = document() else { return };
    let Some((_, trigger)) = parts(&doc) else {
        return;
    };

    let on_trigger = Closure::<dyn FnMut()>::new(toggle);
    let _ = trigger.add_event_listener_with_callback("click", on_trigger.as_ref().unchecked_ref());
    on_trigger.forget(); // lives for the page's lifetime

    if let Ok(Some(button)) = doc.query_selector("#system-close") {
        let on_close = Closure::<dyn FnMut()>::new(close);
        let _ = button.add_event_listener_with_callback("click", on_close.as_ref().unchecked_ref());
        on_close.forget();
    }

    // Article pages have no other keyboard model, and on the portfolio this
    // agrees with the Escape branch that also dismisses the case study.
    let on_key = Closure::<dyn FnMut(KeyboardEvent)>::new(move |event: KeyboardEvent| {
        if event.key() == "Escape" {
            close();
        }
    });
    if let Some(window) = web_sys::window() {
        let _ = window.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref());
        on_key.forget();
    }
}

/// Replaces the deterministic build-time placeholders with what this browser
/// actually did — in the diagnostics panel and in the hero's runtime facts,
/// which state the same three numbers. Cells are addressed by id so neither
/// layout has to be known here, and a missing id is simply skipped.
///
/// Writing to `textContent` after hydration is safe precisely because no signal
/// owns these nodes: the static render and the first hydrated render both emit
/// the placeholder, and only then does the measurement land.
pub fn publish_measurements() {
    let Some(doc) = document() else { return };
    let boot = format!("{:.1} ms", crate::runtime::now_ms());
    let size = crate::runtime::wasm_transfer_size()
        .unwrap_or_else(|| crate::runtime::PRODUCTION_WASM_SIZE.to_string());
    for (id, value) in [
        ("fact-engine", crate::runtime::browser_engine()),
        ("fact-boot", boot.clone()),
        ("fact-wasm", size.clone()),
        ("hero-wasm", format!("{size} WASM")),
        ("hero-boot", format!("BOOT {boot}")),
    ] {
        if let Some(cell) = doc.get_element_by_id(&id) {
            cell.set_text_content(Some(&value));
        }
    }
}
