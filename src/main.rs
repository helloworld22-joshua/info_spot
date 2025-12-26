mod api;
mod components;
mod models;
mod oauth;

use crate::api::SpotifyClient;
use crate::components::{UserProfile, TopTracks, TopArtists, Playlists, RecentlyPlayed};
use crate::models::*;
use dioxus::prelude::*;
use std::rc::Rc;

// Toast notification type
#[derive(Clone, PartialEq)]
enum ToastType {
    Success,
    Error,
    Info,
}

#[derive(Clone, PartialEq)]
struct Toast {
    message: String,
    toast_type: ToastType,
    id: usize,
}

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

    rsx! {
		document::Link { rel: "stylesheet", href: asset!("assets/compiled/main.css") }
		Router::<Route> {}
		ToastContainer {}
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

#[component]
fn Home() -> Element {
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

#[component]
fn Callback() -> Element {
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
		div { class: "callback-container",
			p { "Processing authentication..." }
		}
	}
}

#[component]
fn Dashboard() -> Element {
    let context = use_context::<AppContext>();
    let nav = navigator();

    let mut user = use_signal(|| None::<User>);
    let mut top_tracks = use_signal(|| Vec::<Track>::new());
    let mut top_artists = use_signal(|| Vec::<Artist>::new());
    let mut playlists = use_signal(|| Vec::<Playlist>::new());
    let mut recently_played = use_signal(|| Vec::<RecentlyPlayedItem>::new());
    let mut time_range = use_signal(|| "short_term".to_string());
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);

    // Import playlist modal state
    let mut show_import_modal = use_signal(|| false);
    let mut import_name = use_signal(|| String::new());
    let mut import_description = use_signal(|| String::new());
    let mut import_author = use_signal(|| String::new());
    let mut import_track_uris = use_signal(|| Vec::<String>::new());
    let mut import_tracks = use_signal(|| Vec::<Track>::new());
    let mut importing = use_signal(|| false);
    let mut loading_tracks = use_signal(|| false);

    let is_demo_mode = context.demo_mode.read().clone();

    // Check if we have a Spotify client with token or if we're in demo mode
    let spotify_client_option = context.spotify_client.read().clone();

    if !is_demo_mode && spotify_client_option.is_none() {
        // No authenticated client and not in demo mode, redirect to home
        use_effect(move || {
            nav.push(Route::Home {});
        });

        return rsx! {
			div { class: "loading", "Redirecting to login..." }
		};
    }

    // Load data based on mode
    if is_demo_mode {
        // Demo mode - use mock data
        use_effect(move || {
            user.set(Some(get_mock_user()));
            top_tracks.set(get_mock_top_tracks());
            top_artists.set(get_mock_top_artists());
            playlists.set(get_mock_playlists());
            recently_played.set(get_mock_recently_played());
            loading.set(false);
        });
    } else if let Some(ref client) = spotify_client_option {
        // Real mode - fetch from Spotify API
        // Initial data fetch
        {
            let client_clone = client.clone();
            use_effect(move || {
                let client_clone2 = client_clone.clone();
                let current_time_range = time_range();

                spawn(async move {
                    // Fetch user data
                    match client_clone2.get_current_user().await {
                        Ok(user_data) => {
                            user.set(Some(user_data));
                        }
                        Err(e) => {
                            eprintln!("Failed to fetch user: {}", e);
                            error.set(Some(format!("Failed to fetch user data: {}", e)));
                        }
                    }

                    // Fetch top tracks
                    match client_clone2.get_top_tracks(20, &current_time_range).await {
                        Ok(tracks) => {
                            top_tracks.set(tracks);
                        }
                    Err(e) => {
                        eprintln!("Failed to fetch top tracks: {}", e);
                    }
                }

                // Fetch top artists
                match client_clone2.get_top_artists(20, &current_time_range).await {
                    Ok(artists) => {
                        top_artists.set(artists);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch top artists: {}", e);
                    }
                }

                // Fetch playlists
                match client_clone2.get_playlists(50).await {
                    Ok(user_playlists) => {
                        println!("DEBUG: Fetched {} playlists", user_playlists.len());
                        playlists.set(user_playlists);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch playlists: {}", e);
                    }
                }

                // Fetch recently played (last 50 tracks)
                match client_clone2.get_recently_played(50).await {
                    Ok(recent_tracks) => {
                        println!("DEBUG: Fetched {} recently played tracks", recent_tracks.len());
                        recently_played.set(recent_tracks);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch recently played: {}", e);
                    }
                }

                loading.set(false);
            });
        });
    }
    }

    // Create event handlers for time range buttons
    let on_short_term = {
        let client_opt = spotify_client_option.clone();
        let demo = is_demo_mode;
        move |_| {
            let new_range = "short_term".to_string();
            time_range.set(new_range.clone());

            if demo {
                return; // Just update UI state in demo mode
            }

            if let Some(client) = client_opt.clone() {
                loading.set(true);
                spawn(async move {
                    if let Ok(tracks) = client.get_top_tracks(20, &new_range).await {
                        top_tracks.set(tracks);
                    }

                    if let Ok(artists) = client.get_top_artists(20, &new_range).await {
                        top_artists.set(artists);
                    }

                    loading.set(false);
                });
            }
        }
    };

    let on_medium_term = {
        let client_opt = spotify_client_option.clone();
        let demo = is_demo_mode;
        move |_| {
            let new_range = "medium_term".to_string();
            time_range.set(new_range.clone());

            if demo {
                return;
            }

            if let Some(client) = client_opt.clone() {
                loading.set(true);
                spawn(async move {
                    if let Ok(tracks) = client.get_top_tracks(20, &new_range).await {
                        top_tracks.set(tracks);
                    }

                    if let Ok(artists) = client.get_top_artists(20, &new_range).await {
                        top_artists.set(artists);
                    }

                    loading.set(false);
                });
            }
        }
    };

    let on_long_term = {
        let client_opt = spotify_client_option.clone();
        let demo = is_demo_mode;
        move |_| {
            let new_range = "long_term".to_string();
            time_range.set(new_range.clone());

            if demo {
                return;
            }

            if let Some(client) = client_opt.clone() {
                loading.set(true);
                spawn(async move {
                    if let Ok(tracks) = client.get_top_tracks(20, &new_range).await {
                        top_tracks.set(tracks);
                    }

                    if let Ok(artists) = client.get_top_artists(20, &new_range).await {
                        top_artists.set(artists);
                    }

                    loading.set(false);
                });
            }
        }
    };

    // Import playlist handler
    let on_import_playlist = {
        let demo = is_demo_mode;
        let client_opt = spotify_client_option.clone();
        move |_| {
            if demo {
                println!("Import not available in demo mode");
                return;
            }

            let client_opt = client_opt.clone();
            spawn(async move {
                if let Some(file_path) = pick_json_file() {
                    // Read and parse the JSON file
                    match std::fs::read_to_string(&file_path) {
                        Ok(file_content) => {
                            match serde_json::from_str::<serde_json::Value>(&file_content) {
                                Ok(json_data) => {
                                    // Extract playlist info
                                    let name = json_data["info"]["name"].as_str().unwrap_or("Imported Playlist").to_string();
                                    let description = json_data["info"]["description"].as_str().unwrap_or("").to_string();
                                    let author = json_data["info"]["author"].as_str().unwrap_or("Unknown").to_string();

                                    // Extract track URIs
                                    let track_uris: Vec<String> = json_data["tracks"]
                                        .as_array()
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|t| t.as_str().map(String::from))
                                                .collect()
                                        })
                                        .unwrap_or_default();

                                    if track_uris.is_empty() {
                                        eprintln!("No tracks found in JSON file");
                                        error.set(Some("No tracks found in the selected file".to_string()));
                                        return;
                                    }

                                    // Set modal data
                                    import_name.set(name);
                                    import_description.set(description);
                                    import_author.set(author);
                                    import_track_uris.set(track_uris.clone());
                                    show_import_modal.set(true);
                                    loading_tracks.set(true);

                                    // Fetch track details from Spotify
                                    if let Some(client) = client_opt {
                                        // Extract track IDs from URIs (spotify:track:ID)
                                        let track_ids: Vec<String> = track_uris.iter()
                                            .filter_map(|uri| {
                                                uri.split(':').nth(2).map(String::from)
                                            })
                                            .collect();

                                        match client.get_tracks(track_ids).await {
                                            Ok(tracks) => {
                                                import_tracks.set(tracks);
                                                loading_tracks.set(false);
                                            }
                                            Err(e) => {
                                                eprintln!("Failed to fetch track details: {}", e);
                                                loading_tracks.set(false);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to parse JSON: {}", e);
                                    error.set(Some(format!("Failed to parse JSON file: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read file: {}", e);
                            error.set(Some(format!("Failed to read file: {}", e)));
                        }
                    }
                }
            });
        }
    };

    // Confirm import handler
    let confirm_import = {
        let client_opt = spotify_client_option.clone();
        let ctx = use_context::<AppContext>();
        move |_| {
            if let Some(client) = client_opt.clone() {
                importing.set(true);

                let name = import_name();
                let description = import_description();
                let track_uris = import_track_uris();
                let context = ctx.clone();

                spawn(async move {
                    // Create the playlist
                    match client.create_playlist(&name, &description, false).await {
                        Ok(playlist) => {
                            println!("✓ Created playlist: {}", playlist.name);

                            // Add tracks to the playlist
                            match client.add_tracks_to_playlist(&playlist.id, track_uris).await {
                                Ok(_) => {
                                    println!("✓ Successfully imported playlist!");

                                    // Refresh playlists
                                    match client.get_playlists(50).await {
                                        Ok(updated_playlists) => {
                                            println!("✓ Refreshed playlist list");
                                            playlists.set(updated_playlists);
                                            show_import_modal.set(false);
                                            importing.set(false);
                                            show_success(&context, format!("Successfully imported playlist '{}'", name));
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to refresh playlists: {}", e);
                                            importing.set(false);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to add tracks to playlist: {}", e);
                                    error.set(Some(format!("Failed to add tracks: {}", e)));
                                    show_error(&context, format!("Failed to add tracks: {}", e));
                                    importing.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to create playlist: {}", e);
                            error.set(Some(format!("Failed to create playlist: {}", e)));
                            show_error(&context, format!("Failed to create playlist: {}", e));
                            importing.set(false);
                        }
                    }
                });
            }
        }
    };

    let mut mouse_pos = use_signal(|| (0, 0));
    let mut scroll_pos = use_signal(|| (0, 0));
    rsx! {
		div {
			class: "dashboard-container",
			onmousemove: move |event| {
			    let coords = event.data().client_coordinates();
			    mouse_pos.set((coords.x as i32, coords.y as i32));
			},
			// Inside Dashboard component in main.rs
			onscroll: move |event| {
			    // Get the X and Y offsets separately
			    let x = event.data().scroll_left();
			    let y = event.data().scroll_top();

			    // Update your tuple signal
			    scroll_pos.set((x as i32, y as i32));
			},
			style: "
                --mouse-x: {mouse_pos().0}px;
                --mouse-y: {mouse_pos().1}px;
                --scroll-x: {scroll_pos().0}px;
                --scroll-y: {scroll_pos().1}px;
            ",

			if is_demo_mode {
				div { style: "background: linear-gradient(135deg, var(--secondary) 0%, var(--primary) 100%); padding: 12px 20px; text-align: center; margin-bottom: 20px; border-radius: 8px;",
					p { style: "margin: 0; font-weight: 600; font-size: 0.95rem;",
						"🎭 Demo Mode - Using mock data (no Spotify connection required)"
					}
				}
			}

			header { class: "dashboard-header",
				div { class: "header-row",
					h1 { class: "dashboard-title", "Your Spotify Stats" }
				}
				div { class: "time-range-selector",
					button {
						class: if time_range() == "short_term" { "active" } else { "" },
						onclick: on_short_term,
						"Last 4 Weeks"
					}
					button {
						class: if time_range() == "medium_term" { "active" } else { "" },
						onclick: on_medium_term,
						"Last 6 Months"
					}
					button {
						class: if time_range() == "long_term" { "active" } else { "" },
						onclick: on_long_term,
						"All Time"
					}
				}
			}

			if let Some(err) = error() {
				div { style: "padding: 20px; background: rgba(255,68,68,0.1); border: 1px solid #ff4444; border-radius: 8px; margin: 20px;",
					p { style: "color: #ff4444; margin: 0;", "Error: {err}" }
					p { style: "color: #b3b3b3; margin-top: 10px; font-size: 0.9rem;",
						"Please try logging in again or check the console for details."
					}
				}
			}

			if loading() {
				div { class: "loading", "Loading your data..." }
			} else {
				div { class: "dashboard-content",
					UserProfile { user, on_import: on_import_playlist }
					TopTracks { tracks: top_tracks }
					TopArtists { artists: top_artists }
					Playlists { playlists }
					RecentlyPlayed { recent_tracks: recently_played }
				}
			}

			// Import Playlist Preview Modal
			if show_import_modal() {
				div {
					class: "modal-overlay",
					onclick: move |_| show_import_modal.set(false),
					div {
						class: "modal-content import-modal",
						onclick: move |e| e.stop_propagation(),
						div { class: "modal-header",
							h2 { "Import Playlist" }
							button {
								class: "modal-close",
								onclick: move |_| show_import_modal.set(false),
								"×"
							}
						}

						div { class: "modal-body",
							div { class: "import-form",
								div { class: "form-group",
									label { "Playlist Name" }
									input {
										r#type: "text",
										class: "form-input",
										value: "{import_name()}",
										oninput: move |e| import_name.set(e.value().clone()),
									}
								}

								div { class: "form-group",
									label { "Description" }
									textarea {
										class: "form-textarea",
										value: "{import_description()}",
										oninput: move |e| import_description.set(e.value().clone()),
										rows: "3",
									}
								}

								div { class: "form-group",
									label { "Original Author" }
									p { class: "author-text", "{import_author()}" }
								}

								div { class: "form-group",
									label { "Tracks ({import_track_uris().len()} songs)" }

									if loading_tracks() {
										div { class: "tracks-preview",
											p { style: "text-align: center; padding: 20px; color: var(--text-secondary);",
												"Loading track details..."
											}
										}
									} else if import_tracks().is_empty() {
										div { class: "tracks-preview",
											for (index , track_uri) in import_track_uris().iter().enumerate() {
												div {
													class: "track-preview-item",
													key: "{index}",
													span { class: "track-number", "{index + 1}." }
													span { class: "track-uri", "{track_uri}" }
												}
											}
										}
									} else {
										div { class: "tracks-preview",
											for (index , track) in import_tracks().iter().enumerate() {
												div {
													class: "track-preview-item",
													key: "{index}",
													span { class: "track-number", "{index + 1}." }
													if let Some(image) = track.album.images.first() {
														img {
															class: "track-preview-image",
															src: "{image.url}",
															alt: "{track.name}",
														}
													}
													div { class: "track-preview-info",
														div { class: "track-preview-name",
															"{track.name}"
														}
														div { class: "track-preview-artist",
															{track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")}
														}
													}
												}
											}
										}
									}
								}
							}
						}

						div { class: "modal-footer",
							button {
								class: "modal-button cancel-button",
								onclick: move |_| show_import_modal.set(false),
								"Cancel"
							}
							button {
								class: "modal-button import-confirm-button",
								onclick: confirm_import,
								disabled: importing(),
								if importing() {
									"Importing..."
								} else {
									"Import Playlist"
								}
							}
						}
					}
				}
			}
		}
	}
}

#[component]
fn PlaylistDetail(id: String) -> Element {
    let context = use_context::<AppContext>();
    let nav = navigator();

    let mut tracks = use_signal(|| Vec::<PlaylistTrackItem>::new());
    let mut playlist_info = use_signal(|| None::<Playlist>);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut sort_order = use_signal(|| "default".to_string());
    let mut show_duplicates_modal = use_signal(|| false);
    let mut duplicates = use_signal(|| Vec::<(Track, Vec<usize>)>::new());
    let mut removing_duplicates = use_signal(|| false);

    // Check if we have a Spotify client with token
    let spotify_client_option = context.spotify_client.read().clone();

    if spotify_client_option.is_none() {
        use_effect(move || {
            nav.push(Route::Home {});
        });

        return rsx! {
			div { class: "loading", "Redirecting to login..." }
		};
    }

    let client = spotify_client_option.unwrap();

    // Fetch playlist info and tracks
    {
        let client_clone = client.clone();
        let playlist_id = id.clone();
        use_effect(move || {
            let client_clone2 = client_clone.clone();
            let playlist_id_clone = playlist_id.clone();

            spawn(async move {
                // Fetch playlist details
                match client_clone2.get_playlist(&playlist_id_clone).await {
                    Ok(playlist) => {
                        playlist_info.set(Some(playlist));
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch playlist info: {}", e);
                    }
                }

                // Fetch playlist tracks
                match client_clone2.get_playlist_tracks(&playlist_id_clone).await {
                    Ok(playlist_tracks) => {
                        println!("DEBUG: Fetched {} tracks", playlist_tracks.len());
                        tracks.set(playlist_tracks);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch playlist tracks: {}", e);
                        error.set(Some(format!("Failed to load tracks: {}", e)));
                    }
                }
                loading.set(false);
            });
        });
    }

    // Download playlist as JSON
    let download_json = {
        let ctx = use_context::<AppContext>();
        move |_| {
            let playlist = playlist_info();
            let tracks_list = tracks();
            let context = ctx.clone();

            if let Some(pl) = playlist {
                spawn(async move {
                    // Create JSON structure
                    let json_data = serde_json::json!({
                        "info": {
                            "name": pl.name,
                            "id": pl.id,
                            "author": pl.owner.display_name.unwrap_or_else(|| "Unknown".to_string()),
                            "description": pl.description.unwrap_or_else(|| "".to_string()),
                        },
                        "tracks": tracks_list.iter().map(|item| {
                            format!("spotify:track:{}", item.track.id)
                        }).collect::<Vec<_>>()
                    });

                    // Convert to pretty JSON string
                    match serde_json::to_string_pretty(&json_data) {
                        Ok(json_string) => {
                            // Sanitize filename
                            let filename = sanitize_filename(&pl.name);
                            let default_filename = format!("{}.json", filename);

                            // Show save dialog to let user choose location
                            if let Some(save_path) = save_json_file(&default_filename) {
                                // Ensure the path ends with .json
                                let final_path = if save_path.ends_with(".json") {
                                    save_path
                                } else {
                                    format!("{}.json", save_path)
                                };

                                match std::fs::write(&final_path, &json_string) {
                                    Ok(_) => {
                                        println!("✓ Playlist exported to: {}", final_path);
                                        show_success(&context, format!("Playlist exported successfully to {}",
                                            std::path::Path::new(&final_path)
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("file")));
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to save playlist: {}", e);
                                        show_error(&context, format!("Failed to save playlist: {}", e));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to serialize JSON: {}", e);
                            show_error(&context, format!("Failed to create JSON: {}", e));
                        }
                    }
                });
            }
        }
    };

    // Find and show duplicates
    let find_duplicates = {
        let ctx = use_context::<AppContext>();
        move |_| {
            let tracks_list = tracks();
            let mut track_map: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
            let context = ctx.clone();

            // Group tracks by ID
            for (index, item) in tracks_list.iter().enumerate() {
                track_map.entry(item.track.id.clone())
                    .or_insert_with(Vec::new)
                    .push(index);
            }

            // Find tracks that appear more than once
            let duplicate_tracks: Vec<(Track, Vec<usize>)> = track_map
                .into_iter()
                .filter(|(_, indices)| indices.len() > 1)
                .map(|(track_id, indices)| {
                    // Get the track from the first occurrence
                    let track = tracks_list[indices[0]].track.clone();
                    (track, indices)
                })
                .collect();

            if duplicate_tracks.is_empty() {
                show_info(&context, "No duplicate tracks found in this playlist".to_string());
            } else {
                duplicates.set(duplicate_tracks);
                show_duplicates_modal.set(true);
            }
        }
    };

    // Remove duplicates from playlist
    let remove_duplicates_handler = {
        let ctx = use_context::<AppContext>();
        move |_| {
            let client = client.clone();
            let playlist_id = id.clone();
            let duplicates_list = duplicates();
            let context = ctx.clone();

            removing_duplicates.set(true);

            spawn(async move {
                let mut tracks_to_remove = Vec::new();
                let total_duplicates = duplicates_list.iter()
                    .map(|(_, indices)| indices.len() - 1)
                    .sum::<usize>();

                // For each duplicate track, keep the first occurrence and remove the rest
                for (track, indices) in duplicates_list.iter() {
                    println!("DEBUG: Track '{}' appears at positions: {:?}", track.name, indices);
                    // Skip the first index (keep one copy), add the rest for removal with their positions
                    for &index in indices.iter().skip(1) {
                        println!("DEBUG: Will remove '{}' at position {}", track.name, index);
                        tracks_to_remove.push((format!("spotify:track:{}", track.id), index));
                    }
                }

                // Sort by position in descending order to avoid index shifting issues
                tracks_to_remove.sort_by(|a, b| b.1.cmp(&a.1));

                println!("DEBUG: Total tracks to remove: {}", tracks_to_remove.len());

                if !tracks_to_remove.is_empty() {
                    match client.remove_tracks_from_playlist(&playlist_id, tracks_to_remove).await {
                        Ok(_) => {
                            println!("✓ Successfully removed duplicates");
                            // Refresh the track list
                            match client.get_playlist_tracks(&playlist_id).await {
                                Ok(updated_tracks) => {
                                    tracks.set(updated_tracks);
                                    show_duplicates_modal.set(false);
                                    duplicates.set(Vec::new());
                                    show_success(&context, format!("Successfully removed {} duplicate track(s)", total_duplicates));
                                }
                                Err(e) => {
                                    eprintln!("Failed to refresh tracks: {}", e);
                                    show_error(&context, format!("Failed to refresh playlist: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to remove duplicates: {}", e);
                            error.set(Some(format!("Failed to remove duplicates: {}", e)));
                            show_error(&context, format!("Failed to remove duplicates: {}", e));
                        }
                    }
                }

                removing_duplicates.set(false);
            });
        }
    };

    // Sort tracks based on selected order
    let sorted_tracks = {
        let mut tracks_vec = tracks();
        match sort_order().as_str() {
            "name_asc" => tracks_vec.sort_by(|a, b| a.track.name.to_lowercase().cmp(&b.track.name.to_lowercase())),
            "name_desc" => tracks_vec.sort_by(|a, b| b.track.name.to_lowercase().cmp(&a.track.name.to_lowercase())),
            "artist_asc" => tracks_vec.sort_by(|a, b| {
                let a_artist = a.track.artists.first().map(|ar| ar.name.to_lowercase()).unwrap_or_default();
                let b_artist = b.track.artists.first().map(|ar| ar.name.to_lowercase()).unwrap_or_default();
                a_artist.cmp(&b_artist)
            }),
            "artist_desc" => tracks_vec.sort_by(|a, b| {
                let a_artist = a.track.artists.first().map(|ar| ar.name.to_lowercase()).unwrap_or_default();
                let b_artist = b.track.artists.first().map(|ar| ar.name.to_lowercase()).unwrap_or_default();
                b_artist.cmp(&a_artist)
            }),
            "date_added_desc" => tracks_vec.sort_by(|a, b| b.added_at.cmp(&a.added_at)),
            "date_added_asc" => tracks_vec.sort_by(|a, b| a.added_at.cmp(&b.added_at)),
            _ => {} // default - keep original order
        }
        tracks_vec
    };

    rsx! {
		div { class: "playlist-detail-container",
			header { class: "playlist-detail-header",
				div { class: "header-actions",
					button {
						class: "back-button",
						onclick: move |_| {
						    nav.push(Route::Dashboard {});
						},
						"← Back to Dashboard"
					}
					if playlist_info().is_some() {
						button { class: "download-button", onclick: download_json, "⬇ Download JSON" }
						button {
							class: "remove-duplicates-button",
							onclick: find_duplicates,
							style: "margin-left: 10px;",
							"🔍 Remove Duplicates"
						}
					}
				}
				h1 { class: "playlist-detail-title",
					{
					    playlist_info()
					        .as_ref()
					        .map(|p| p.name.clone())
					        .unwrap_or_else(|| "Playlist Tracks".to_string())
					}
				}
			}

			div { class: "sort-controls",
				span { class: "sort-label", "Sort by:" }
				button {
					class: if sort_order() == "default" { "sort-button active" } else { "sort-button" },
					onclick: move |_| sort_order.set("default".to_string()),
					"Default Order"
				}
				button {
					class: if sort_order() == "name_asc" { "sort-button active" } else { "sort-button" },
					onclick: move |_| sort_order.set("name_asc".to_string()),
					"Name A-Z"
				}
				button {
					class: if sort_order() == "name_desc" { "sort-button active" } else { "sort-button" },
					onclick: move |_| sort_order.set("name_desc".to_string()),
					"Name Z-A"
				}
				button {
					class: if sort_order() == "artist_asc" { "sort-button active" } else { "sort-button" },
					onclick: move |_| sort_order.set("artist_asc".to_string()),
					"Artist A-Z"
				}
				button {
					class: if sort_order() == "artist_desc" { "sort-button active" } else { "sort-button" },
					onclick: move |_| sort_order.set("artist_desc".to_string()),
					"Artist Z-A"
				}
				button {
					class: if sort_order() == "date_added_desc" { "sort-button active" } else { "sort-button" },
					onclick: move |_| sort_order.set("date_added_desc".to_string()),
					"Newest First"
				}
				button {
					class: if sort_order() == "date_added_asc" { "sort-button active" } else { "sort-button" },
					onclick: move |_| sort_order.set("date_added_asc".to_string()),
					"Oldest First"
				}
			}

			if let Some(err) = error() {
				div { class: "error-message",
					p { "Error: {err}" }
				}
			}

			if loading() {
				div { class: "loading", "Loading tracks..." }
			} else {
				div { class: "tracks-list",
					for (index , item) in sorted_tracks.iter().enumerate() {
						div {
							class: "track-item",
							key: "{item.track.id}-{index}",
							span { class: "track-number", "{index + 1}" }
							if let Some(image) = item.track.album.images.first() {
								img {
									class: "track-image",
									src: "{image.url}",
									alt: "{item.track.name}",
								}
							}
							div { class: "track-info",
								a {
									class: "track-name",
									href: "{item.track.external_urls.spotify}",
									target: "_blank",
									"{item.track.name}"
								}
								div { class: "track-artists",
									{item.track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")}
								}
							}
							div { class: "track-duration", {format_duration(item.track.duration_ms)} }
						}
					}
				}
			}

			// Duplicates Modal
			if show_duplicates_modal() {
				div {
					class: "modal-overlay",
					onclick: move |_| show_duplicates_modal.set(false),
					div {
						class: "modal-content",
						onclick: move |e| e.stop_propagation(),
						div { class: "modal-header",
							h2 { "Duplicate Tracks Found" }
							button {
								class: "modal-close",
								onclick: move |_| show_duplicates_modal.set(false),
								"×"
							}
						}

						div { class: "modal-body",
							if duplicates().is_empty() {
								p { style: "text-align: center; padding: 20px;",
									"No duplicate tracks found in this playlist!"
								}
							} else {
								p { style: "margin-bottom: 15px;",
									"Found {duplicates().len()} duplicate track(s). The first occurrence will be kept."
								}

								div { class: "duplicates-list",
									for (track , indices) in duplicates().iter() {
										div { class: "duplicate-item",
											if let Some(image) = track.album.images.first() {
												img {
													class: "duplicate-image",
													src: "{image.url}",
													alt: "{track.name}",
												}
											}
											div { class: "duplicate-info",
												div { class: "duplicate-name", "{track.name}" }
												div { class: "duplicate-artist",
													{track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")}
												}
												div { class: "duplicate-count",
													"Appears {indices.len()} times (will remove {indices.len() - 1})"
												}
											}
										}
									}
								}
							}
						}

						div { class: "modal-footer",
							button {
								class: "modal-button cancel-button",
								onclick: move |_| show_duplicates_modal.set(false),
								"Cancel"
							}
							if !duplicates().is_empty() {
								button {
									class: "modal-button remove-button",
									onclick: remove_duplicates_handler,
									disabled: removing_duplicates(),
									if removing_duplicates() {
										"Removing..."
									} else {
										"Remove Duplicates"
									}
								}
							}
						}
					}
				}
			}
		}
	}
}

fn format_duration(ms: u32) -> String {
    let total_seconds = ms / 1000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

fn pick_json_file() -> Option<String> {
    use std::process::Command;

    // Use native file picker based on OS
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"POSIX path of (choose file with prompt "Select a playlist JSON file" of type {"public.json"})"#)
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            let trimmed = path.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("zenity")
            .args(&["--file-selection", "--file-filter=*.json"])
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            return Some(path.trim().to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // For Windows, we'll use a simple dialog
        println!("Please enter the full path to the JSON file:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;
        return Some(input.trim().to_string());
    }

    None
}

// Toast helper functions
fn show_toast(context: &AppContext, message: String, toast_type: ToastType) {
    let mut toasts = context.toasts;
    let mut counter = context.toast_counter;

    let id = counter();
    counter.set(id + 1);

    let toast = Toast {
        message,
        toast_type,
        id,
    };

    let mut current_toasts = toasts();
    current_toasts.push(toast.clone());
    toasts.set(current_toasts);

    // Auto-remove toast after 5 seconds
    spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        let mut current_toasts = toasts();
        current_toasts.retain(|t| t.id != id);
        toasts.set(current_toasts);
    });
}

fn show_success(context: &AppContext, message: String) {
    show_toast(context, message, ToastType::Success);
}

fn show_error(context: &AppContext, message: String) {
    show_toast(context, message, ToastType::Error);
}

fn show_info(context: &AppContext, message: String) {
    show_toast(context, message, ToastType::Info);
}

#[component]
fn ToastContainer() -> Element {
    let context = use_context::<AppContext>();
    let toasts = context.toasts;

    rsx! {
		div { class: "toast-container",
			for toast in toasts().iter() {
				ToastItem { key: "{toast.id}", toast: toast.clone() }
			}
		}
	}
}

#[component]
fn ToastItem(toast: Toast) -> Element {
    let mut context = use_context::<AppContext>();
    let toast_id = toast.id;

    let class_name = match toast.toast_type {
        ToastType::Success => "toast toast-success",
        ToastType::Error => "toast toast-error",
        ToastType::Info => "toast toast-info",
    };

    let icon = match toast.toast_type {
        ToastType::Success => "✓",
        ToastType::Error => "✕",
        ToastType::Info => "ℹ",
    };

    let remove_toast = move |_| {
        let mut current_toasts = context.toasts.write();
        *current_toasts = current_toasts.iter().filter(|t| t.id != toast_id).cloned().collect();
    };

    rsx! {
		div { class: "{class_name}",
			span { class: "toast-icon", "{icon}" }
			span { class: "toast-message", "{toast.message}" }
			button { class: "toast-close", onclick: remove_toast, "×" }
		}
	}
}

fn save_json_file(default_filename: &str) -> Option<String> {
    use std::process::Command;

    // Use native save dialog based on OS
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            r#"POSIX path of (choose file name with prompt "Save playlist as" default name "{}")"#,
            default_filename
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            let trimmed = path.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("zenity")
            .args(&[
                "--file-selection",
                "--save",
                "--confirm-overwrite",
                &format!("--filename={}", default_filename),
                "--file-filter=*.json"
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            return Some(path.trim().to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // For Windows, we'll use a simple dialog
        println!("Enter the full path where you want to save {} (or press Enter for default):", default_filename);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;
        let path = input.trim();

        if path.is_empty() {
            // Use default Downloads location
            let home_dir = std::env::var("USERPROFILE").ok()?;
            return Some(format!("{}\\Downloads\\{}", home_dir, default_filename));
        }

        return Some(path.to_string());
    }

    None
}

async fn import_playlist(client: Rc<SpotifyClient>, file_path: String) {
    println!("Importing playlist from: {}", file_path);

    // Read the JSON file
    let file_content = match std::fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            return;
        }
    };

    // Parse JSON
    let json_data: serde_json::Value = match serde_json::from_str(&file_content) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            return;
        }
    };

    // Extract playlist info
    let name = json_data["info"]["name"].as_str().unwrap_or("Imported Playlist");
    let description = json_data["info"]["description"].as_str().unwrap_or("");

    // Extract track URIs
    let tracks: Vec<String> = json_data["tracks"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if tracks.is_empty() {
        eprintln!("No tracks found in JSON file");
        return;
    }

    println!("Creating playlist '{}' with {} tracks...", name, tracks.len());

    // Create the playlist
    match client.create_playlist(name, description, false).await {
        Ok(playlist) => {
            println!("✓ Created playlist: {}", playlist.name);

            // Add tracks to the playlist
            match client.add_tracks_to_playlist(&playlist.id, tracks).await {
                Ok(_) => {
                    println!("✓ Successfully imported playlist!");
                    println!("  View it in Spotify: {}", playlist.external_urls.spotify);
                }
                Err(e) => {
                    eprintln!("Failed to add tracks to playlist: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to create playlist: {}", e);
        }
    }
}

fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

// Mock data generation functions for demo mode
fn get_mock_user() -> User {
    User {
        display_name: Some("Demo User".to_string()),
        id: "demo_user_123".to_string(),
        email: Some("demo@example.com".to_string()),
        images: vec![],
        followers: Followers { total: 42 },
        country: Some("US".to_string()),
    }
}

fn get_mock_top_tracks() -> Vec<Track> {
    vec![
        Track {
            id: "track1".to_string(),
            name: "Bohemian Rhapsody".to_string(),
            artists: vec![Artist {
                id: "artist1".to_string(),
                name: "Queen".to_string(),
                genres: None,
                images: None,
                external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
                followers: None,
            }],
            album: Album {
                id: "album1".to_string(),
                name: "A Night at the Opera".to_string(),
                images: vec![],
                release_date: "1975-11-21".to_string(),
            },
            duration_ms: 354000,
            external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
        },
        Track {
            id: "track2".to_string(),
            name: "Stairway to Heaven".to_string(),
            artists: vec![Artist {
                id: "artist2".to_string(),
                name: "Led Zeppelin".to_string(),
                genres: None,
                images: None,
                external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
                followers: None,
            }],
            album: Album {
                id: "album2".to_string(),
                name: "Led Zeppelin IV".to_string(),
                images: vec![],
                release_date: "1971-11-08".to_string(),
            },
            duration_ms: 482000,
            external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
        },
        Track {
            id: "track3".to_string(),
            name: "Hotel California".to_string(),
            artists: vec![Artist {
                id: "artist3".to_string(),
                name: "Eagles".to_string(),
                genres: None,
                images: None,
                external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
                followers: None,
            }],
            album: Album {
                id: "album3".to_string(),
                name: "Hotel California".to_string(),
                images: vec![],
                release_date: "1976-12-08".to_string(),
            },
            duration_ms: 391000,
            external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
        },
    ]
}

fn get_mock_top_artists() -> Vec<Artist> {
    vec![
        Artist {
            id: "artist1".to_string(),
            name: "Queen".to_string(),
            genres: Some(vec!["classic rock".to_string(), "glam rock".to_string()]),
            images: Some(vec![]),
            external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
            followers: Some(Followers { total: 35000000 }),
        },
        Artist {
            id: "artist2".to_string(),
            name: "Led Zeppelin".to_string(),
            genres: Some(vec!["hard rock".to_string(), "classic rock".to_string()]),
            images: Some(vec![]),
            external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
            followers: Some(Followers { total: 28000000 }),
        },
        Artist {
            id: "artist3".to_string(),
            name: "Eagles".to_string(),
            genres: Some(vec!["classic rock".to_string(), "soft rock".to_string()]),
            images: Some(vec![]),
            external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
            followers: Some(Followers { total: 15000000 }),
        },
    ]
}

fn get_mock_playlists() -> Vec<Playlist> {
    vec![
        Playlist {
            id: "playlist1".to_string(),
            name: "Classic Rock Favorites".to_string(),
            description: Some("The best of classic rock".to_string()),
            images: vec![],
            tracks: PlaylistTracks { total: 25 },
            external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
            owner: PlaylistOwner {
                display_name: Some("Demo User".to_string()),
                id: "demo_user_123".to_string(),
            },
            public: Some(true),
        },
        Playlist {
            id: "playlist2".to_string(),
            name: "Road Trip Mix".to_string(),
            description: Some("Perfect songs for long drives".to_string()),
            images: vec![],
            tracks: PlaylistTracks { total: 40 },
            external_urls: ExternalUrls { spotify: "https://open.spotify.com".to_string() },
            owner: PlaylistOwner {
                display_name: Some("Demo User".to_string()),
                id: "demo_user_123".to_string(),
            },
            public: Some(false),
        },
    ]
}

fn get_mock_recently_played() -> Vec<RecentlyPlayedItem> {
    let tracks = get_mock_top_tracks();
    vec![
        RecentlyPlayedItem {
            track: tracks[0].clone(),
            played_at: "2024-12-26T10:30:00Z".to_string(),
        },
        RecentlyPlayedItem {
            track: tracks[1].clone(),
            played_at: "2024-12-26T09:15:00Z".to_string(),
        },
    ]
}