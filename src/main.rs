mod api;
mod components;
mod models;
mod oauth;
mod utils;

use crate::api::SpotifyClient;
use crate::components::{Toast, ToastContainer, Callback, Home, Dashboard, PlaylistDetail};
use dioxus::prelude::*;
use std::rc::Rc;

// Global context for sharing Spotify client across routes
#[derive(Clone)]
struct AppContext {
    spotify_client: Signal<Option<Rc<SpotifyClient>>>,
    demo_mode: Signal<bool>,
    toasts: Signal<Vec<Toast>>,
    toast_counter: Signal<usize>,
}

fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Initialize global Spotify client context
    use_context_provider(|| AppContext {
        spotify_client: Signal::new(None),
        demo_mode: Signal::new(false),
        toasts: Signal::new(Vec::new()),
        toast_counter: Signal::new(0),
    });

    let context = use_context::<AppContext>();

    rsx! {
		document::Link { rel: "stylesheet", href: asset!("assets/compiled/main.css") }
		Router::<Route> {}
		ToastContainer { toasts: context.toasts }
	}
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/callback")]
    Callback {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/playlist/:id")]
    PlaylistDetail { id: String },
}