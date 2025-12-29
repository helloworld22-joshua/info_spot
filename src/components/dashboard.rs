use crate::components::{UserProfile, TopTracks, TopArtists, Playlists, RecentlyPlayed};
use crate::models::*;
use crate::{Route, AppContext};
use crate::utils::*;
use dioxus::prelude::*;

#[component]
pub fn Dashboard() -> Element {
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
                    // Fetch user data (only if not already loaded)
                    if user().is_none() {
                        match client_clone2.get_current_user().await {
                            Ok(user_data) => {
                                user.set(Some(user_data));
                            }
                            Err(e) => {
                                eprintln!("Failed to fetch user: {}", e);
                                error.set(Some(format!("Failed to fetch user data: {}", e)));
                            }
                        }
                    } else {
                        println!("DEBUG: Using cached user data");
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

                // Fetch playlists (only if not already loaded)
                if playlists().is_empty() {
                    match client_clone2.get_playlists(50).await {
                        Ok(user_playlists) => {
                            println!("DEBUG: Fetched {} playlists", user_playlists.len());
                            playlists.set(user_playlists);
                        }
                        Err(e) => {
                            eprintln!("Failed to fetch playlists: {}", e);
                        }
                    }
                } else {
                    println!("DEBUG: Using cached playlists ({} items)", playlists().len());
                }

                // Fetch recently played (only if not already loaded)
                if recently_played().is_empty() {
                    match client_clone2.get_recently_played(50).await {
                        Ok(recent_tracks) => {
                            println!("DEBUG: Fetched {} recently played tracks", recent_tracks.len());
                            recently_played.set(recent_tracks);
                        }
                        Err(e) => {
                            eprintln!("Failed to fetch recently played: {}", e);
                        }
                    }
                } else {
                    println!("DEBUG: Using cached recently played ({} items)", recently_played().len());
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

    let mut mouse_pos = use_signal(|| (-500, -500)); // -500 so the effect doesn't appear initially
    let mut scroll_pos = use_signal(|| (0, 0));
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/compiled/user_profile.css") }
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
						"Demo Mode (no Spotify connection required)"
					}
				}
			}

			header { class: "dashboard-header",
				div { class: "header-row",
                    img {
                        class: "dashboard-logo",
                        src: asset!("assets/media/logo.svg"),
                        alt: "InfoSpot Logo"
                    }
                    h1 { class: "dashboard-title", "InfoSpot" }
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
					UserProfile {
					user,
					on_import: on_import_playlist,
					time_range: time_range,
				}
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