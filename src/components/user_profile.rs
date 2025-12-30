use crate::models::User;
use crate::utils::format_country;
use crate::{Route, AppContext};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaFileArrowUp, FaRightFromBracket, FaArrowUpRightFromSquare};
use dioxus_free_icons::Icon;

#[component]
pub fn UserProfile(
    user: ReadSignal<Option<User>>,
    on_import: EventHandler<()>,
    time_range: ReadSignal<String>,
) -> Element {
    let context = use_context::<AppContext>();
    let nav = navigator();
    let mut position = use_signal(|| (0.0, 0.0));
    let mut top_genres = use_signal(|| Vec::<String>::new());

    // Fetch top genres when component mounts or time range changes
    use_effect(move || {
        spawn(async move {
            if let Some(client) = context.spotify_client.read().as_ref() {
                // Get top 50 artists for better genre coverage
                if let Ok(artists) = client.get_top_artists(50, &time_range()).await {
                    // Extract and count genres
                    let mut genre_count: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

                    for artist in artists {
                        if let Some(genres) = artist.genres {
                            for genre in genres {
                                *genre_count.entry(genre).or_insert(0) += 1;
                            }
                        }
                    }

                    // Sort by count and take top 10
                    let mut genres: Vec<(String, usize)> = genre_count.into_iter().collect();
                    genres.sort_by(|a, b| b.1.cmp(&a.1));

                    let top_10: Vec<String> = genres.into_iter()
                        .take(10)
                        .map(|(genre, _)| genre)
                        .collect();

                    top_genres.set(top_10);
                }
            }
        });
    });

rsx! {
	document::Link { rel: "stylesheet", href: asset!("assets/compiled/user_profile.css") }
	div {
		class: "user-profile component",
		onmounted: move |event| {
		    spawn(async move {
		        if let Ok(rect) = event.get_client_rect().await {
		            position.set((rect.origin.x, rect.origin.y));
		        }
		    });
		},
		style: "--position-x: {position().0}px; --position-y: {position().1}px;",
		if let Some(user_data) = user() {
			div { class: "profile-card",
				if let Some(image) = user_data.images.first() {
					img {
						class: "profile-image",
						src: "{image.url}",
						alt: "Profile",
					}
				} else {
					// Placeholder for users without profile image
					div {
						class: "profile-image-placeholder",
						{user_data.display_name
							.as_ref()
							.and_then(|name| name.chars().next())
							.map(|c| c.to_uppercase().to_string())
							.unwrap_or_else(|| "?".to_string())}
					}
				}
				div { class: "profile-info",
					h1 { class: "profile-name",
						{user_data.display_name.clone().unwrap_or_else(|| "Unknown User".to_string())}
					}
					div { class: "profile-stats",
						div { class: "stat",
							span { class: "stat-value", "{user_data.followers.total}" }
							span { class: "stat-label", "Followers" }
						}
						if let Some(country) = &user_data.country {
							div { class: "stat",
								span { class: "stat-value", "{format_country(country)}" }
								span { class: "stat-label", "Country" }
							}
						}
					}
				}
				div { class: "profile-actions",
					button {
						class: "import-button button",
						onclick: move |_| on_import.call(()),
						Icon {
							icon: FaFileArrowUp,
							width: 18,
							height: 18,
						}
						"Import Playlist"
					}
					button {
						class: "spotify-button button",
						onclick: move |_| {
						    let url = format!("https://open.spotify.com/user/{}", user_data.id);
						    let _ = open::that(url);
						},
						Icon {
							icon: FaArrowUpRightFromSquare,
							width: 18,
							height: 18,
						}
						"Go to Spotify"
					}
					button {
						class: "logout-button button",
						onclick: {
						    let mut ctx = context.clone();
						    let nav_clone = nav.clone();
						    move |_| {
						        // Clear the authenticated session
						        ctx.spotify_client.set(None);
						        // Reset demo mode flag
						        ctx.demo_mode.set(false);
						        // Navigate back to login screen
						        nav_clone.push(Route::Home {});
						    }
						},
						Icon {
							icon: FaRightFromBracket,
							width: 18,
							height: 18,
						}
						"Logout"
					}
				}
			}

			// Top Genres Section
			div { class: "top-genres-section",
				h3 { class: "genres-title", "Top Genres" }

				if top_genres().is_empty() {
					div { class: "genres-loading", "Loading genres..." }
				} else {
					div { class: "genres-list",
						for (index, genre) in top_genres().iter().enumerate() {
							div {
								class: "genre-tag",
								key: "{genre}-{index}",
								span { class: "genre-rank", "#{index + 1}" }
								span { class: "genre-name", "{genre}" }
							}
						}
					}
				}
			}
		} else {
			div { class: "loading", "Loading user data..." }
		}
	}
}
}