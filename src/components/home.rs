use crate::api::SpotifyClient;
use crate::{Route, AppContext};
use crate::utils::generate_random_string;
use crate::oauth;
use dioxus::prelude::*;
use std::rc::Rc;

#[component]
pub fn Home() -> Element {
    let nav = navigator();
    let mut context = use_context::<AppContext>();

    let client_id = std::env::var("SPOTIFY_CLIENT_ID")
        .unwrap_or_else(|_| "".to_string());
    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
        .unwrap_or_else(|_| "".to_string());
    let redirect_uri = std::env::var("SPOTIFY_REDIRECT_URI")
        .unwrap_or_else(|_| "http://127.0.0.1:8888/callback".to_string());

    // Check if credentials are configured
    let credentials_missing = client_id.is_empty() || client_secret.is_empty();

    if credentials_missing {
        eprintln!("ERROR: Spotify credentials not found!");
        eprintln!("Please create a .env file with:");
        eprintln!("  SPOTIFY_CLIENT_ID=your_client_id");
        eprintln!("  SPOTIFY_CLIENT_SECRET=your_client_secret");
        eprintln!("  SPOTIFY_REDIRECT_URI=http://127.0.0.1:8888/callback");
    }

    let mut error_msg = use_signal(|| None::<String>);
    let mut authenticating = use_signal(|| false);

    let handle_demo_mode = move |_| {
        context.demo_mode.set(true);
        nav.push(Route::Dashboard {});
    };

    let handle_login = move |_| {
        // Check credentials before attempting login
        if client_id.is_empty() || client_secret.is_empty() {
            error_msg.set(Some(
                "Missing Spotify credentials! Please check your .env file.".to_string()
            ));
            return;
        }

        authenticating.set(true);
        error_msg.set(None);

        let client_id_clone = client_id.clone();
        let client_secret_clone = client_secret.clone();
        let redirect_uri_clone = redirect_uri.clone();
        let nav_clone = nav.clone();

        spawn(async move {
            let spotify_client = SpotifyClient::new(
                client_id_clone,
                client_secret_clone,
                redirect_uri_clone
            );

            let state = generate_random_string(16);
            let auth_url = spotify_client.get_auth_url(&state);

            // Open browser
            if let Err(e) = open::that(&auth_url) {
                eprintln!("Failed to open browser: {}", e);
                error_msg.set(Some(format!("Failed to open browser: {}", e)));
                authenticating.set(false);
                return;
            }

            // Wait for callback
            match oauth::start_callback_server() {
                Ok(code) => {
                    // Exchange code for token
                    match spotify_client.exchange_code(&code).await {
                        Ok(token_response) => {
                            // Store token
                            spotify_client.set_token(token_response.access_token).await;

                            // Store client in context
                            context.spotify_client.set(Some(Rc::new(spotify_client)));

                            // Navigate to dashboard
                            nav_clone.push(Route::Dashboard {});
                        }
                        Err(e) => {
                            error_msg.set(Some(format!("Authentication failed: {}", e)));
                            authenticating.set(false);
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("Callback error: {}", e)));
                    authenticating.set(false);
                }
            }
        });
    };

    rsx! {
		document::Link { rel: "stylesheet", href: asset!("assets/compiled/main.css") }
		document::Link { rel: "stylesheet", href: asset!("assets/compiled/home.css") }
		div { class: "home-container",
			div { class: "login-card",
				h1 { class: "app-title", "InfoSpot" }
				p { class: "app-description",
					"Connect with Spotify to view your music stats and playlists"
				}

				if credentials_missing {
					div { style: "background: #ff4444; padding: 15px; border-radius: 8px; margin-bottom: 20px;",
						p { style: "margin-bottom: 10px; font-weight: bold;",
							"⚠️ Missing Spotify Credentials"
						}
						p { style: "font-size: 0.9rem; margin-bottom: 5px;",
							"Please create a .env file in the project root with:"
						}
						pre { style: "background: rgba(0,0,0,0.3); padding: 10px; border-radius: 4px; font-size: 0.85rem; overflow-x: auto;",
							"SPOTIFY_CLIENT_ID=your_client_id_here\n"
							"SPOTIFY_CLIENT_SECRET=your_client_secret_here\n"
							"SPOTIFY_REDIRECT_URI=http://127.0.0.1:8888/callback"
						}
						p { style: "font-size: 0.85rem; margin-top: 10px;",
							"Get your credentials at: "
							a {
								href: "https://developer.spotify.com/dashboard",
								target: "_blank",
								style: "color: #1ed760; text-decoration: underline;",
								"Spotify Developer Dashboard"
							}
						}
					}
				}

				if authenticating() {
					p { class: "loading", "Authenticating..." }
				} else {
					button {
						class: "login-button",
						onclick: handle_login,
						disabled: credentials_missing,
						style: if credentials_missing { "opacity: 0.5; cursor: not-allowed;" } else { "" },
						"Login with Spotify"
					}
					button {
						class: "demo-button",
						onclick: handle_demo_mode,
						style: "margin-top: 15px;",
						"Demo Mode"
					}
				}

				if let Some(error) = error_msg() {
					p { style: "color: #ff4444; margin-top: 20px;", "{error}" }
				}
			}
		}
	}
}