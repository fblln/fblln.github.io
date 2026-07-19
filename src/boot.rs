//! Browser-only mount and boot-overlay lifecycle.
//!
//! The static renderer must stay free of DOM timing concerns, so this module
//! owns the client hand-off after the hydrated application is ready to paint.

use leptos::prelude::*;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::HtmlElement;

const BOOT_MIN_MS: f64 = 1100.0;

pub fn run() {
    console_error_panic_hook::set_once();
    if crate::articles::is_article_page() {
        crate::articles::enhance();
        return;
    }
    let boot_time = crate::runtime::now_ms();
    #[cfg(feature = "hydrate")]
    if let Some(app_root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("app"))
        .and_then(|element| element.dyn_into::<HtmlElement>().ok())
    {
        leptos::mount::hydrate_from(app_root, || view! { <crate::App /> }).forget();
    } else {
        leptos::mount::mount_to_body(|| view! { <crate::App /> });
    }
    #[cfg(not(feature = "hydrate"))]
    leptos::mount::mount_to_body(|| view! { <crate::App /> });
    scroll_to_initial_fragment();
    reveal_site(boot_time);
}

/// Hydration occurs after the browser's native anchor jump, so retry the jump
/// once the sections exist and cross-surface links retain their destination.
fn scroll_to_initial_fragment() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(id) = window
        .location()
        .hash()
        .ok()
        .and_then(|hash| fragment_id(&hash).map(str::to_owned))
    else {
        return;
    };
    if let Some(target) = window
        .document()
        .and_then(|document| document.get_element_by_id(&id))
    {
        target.scroll_into_view();
    }
}

fn fragment_id(hash: &str) -> Option<&str> {
    hash.strip_prefix('#').filter(|id| !id.is_empty())
}

/// Holds the cold-load runtime screen long enough to be perceived, but removes
/// it immediately on warm loads that the document shell hid before first paint.
fn reveal_site(elapsed_ms: f64) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let document = window.document();
    if let Ok(Some(storage)) = window.local_storage() {
        let _ = storage.set_item("fe:booted", "1");
    }
    let warm = document
        .as_ref()
        .and_then(|d| d.document_element())
        .is_some_and(|el| el.has_attribute("data-warm"));
    let Some(boot) = document.and_then(|d| d.get_element_by_id("boot")) else {
        return;
    };
    if warm {
        boot.remove();
        return;
    }
    let fade = Closure::once_into_js(move || {
        let _ = boot.set_attribute("class", "boot boot-hide");
        let remove = Closure::once_into_js(move || boot.remove());
        if let Some(window) = web_sys::window() {
            let _ = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(remove.unchecked_ref(), 460);
        }
    });
    let hold = (BOOT_MIN_MS - elapsed_ms).max(0.0) as i32;
    let _ =
        window.set_timeout_with_callback_and_timeout_and_arguments_0(fade.unchecked_ref(), hold);
}

#[cfg(test)]
mod tests {
    use super::fragment_id;
    #[test]
    fn fragment_ids_keep_only_non_empty_hash_targets() {
        assert_eq!(fragment_id("#experience"), Some("experience"));
        assert_eq!(fragment_id("#"), None);
        assert_eq!(fragment_id("experience"), None);
    }
}
