use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Callback() -> Element {
    let nav = navigator();

    // Note: In a real implementation, you'd need to handle the OAuth callback
    // This is a simplified version - you'll need to implement a local server
    // to capture the authorization code from the redirect

    use_effect(move || {
        spawn(async move {
            // This is where you'd extract the code from the URL and exchange it
            // For now, this is a placeholder
            nav.push(Route::Dashboard {});
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/compiled/callback.css") }
        div { class: "callback-container",
            p { "Processing authentication..." }
        }
    }
}