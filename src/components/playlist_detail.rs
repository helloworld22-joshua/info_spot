use crate::components::TrackDetail;
use crate::models::*;
use crate::{Route, AppContext};
use crate::utils::*;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaFileArrowDown;
use dioxus_free_icons::icons::fa_solid_icons::FaMagnifyingGlass;
use dioxus_free_icons::Icon;

#[component]
pub fn PlaylistDetail(id: String) -> Element {
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
    let mut selected_track = use_signal(|| None::<Track>);

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
                .map(|(_track_id, indices)| {
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
						button { class: "download-button", onclick: download_json,
                        Icon {
							icon: FaFileArrowDown,
							width: 18,
							height: 18,
						}
                        "Download JSON" }
						button {
							class: "remove-duplicates-button",
							onclick: find_duplicates,
							style: "margin-left: 10px;",
                            Icon {
							icon: FaMagnifyingGlass,
							width: 18,
							height: 18,
						}
							"Remove Duplicates"
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
							class: "track-item clickable",
							key: "{item.track.id}-{index}",
							onclick: {
								let track = item.track.clone();
								move |_| selected_track.set(Some(track.clone()))
							},
							span { class: "track-number", "{index + 1}" }
							if let Some(image) = item.track.album.images.first() {
								img {
									class: "track-image",
									src: "{image.url}",
									alt: "{item.track.name}",
								}
							}
							div { class: "track-info",
								div { class: "track-name", "{item.track.name}" }
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

		// Track detail modal
		if let Some(track) = selected_track() {
			TrackDetail {
				track: track,
				on_close: move |_| selected_track.set(None),
			}
		}
	}
}