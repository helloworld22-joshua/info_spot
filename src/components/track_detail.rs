use crate::models::Track;
use crate::utils::{format_duration, format_release_date};
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaXmark;
use dioxus_free_icons::icons::fa_brands_icons::FaSpotify;
use dioxus_free_icons::Icon;

#[component]
pub fn TrackDetail(track: Track, on_close: EventHandler<()>) -> Element {
    let spotify_url = track.external_urls.spotify.clone();
    let track_uri = format!("spotify:track:{}", track.id);
    let artists = track.artists.iter()
        .map(|a| a.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let duration = format_duration(track.duration_ms);
    let popularity = track.popularity.unwrap_or(0);
    let release_date = format_release_date(&track.album.release_date);

    rsx! {
		document::Link {
			rel: "stylesheet",
			href: asset!("assets/compiled/detail.css"),
		}
		div {
			class: "detail-overlay",
			onclick: move |_| on_close.call(()),

			div {
				class: "detail-modal",
				onclick: move |e| e.stop_propagation(),

				// Close button
				button {
					class: "detail-close",
					onclick: move |_| on_close.call(()),
					Icon { icon: FaXmark, width: 20, height: 20 }
				}

				// Album cover
				if let Some(image) = track.album.images.first() {
					img {
						class: "detail-cover",
						src: "{image.url}",
						alt: "{track.name}",
					}
				}

				// Track info
				div { class: "detail-info",
					div { class: "detail-header",
						h2 { class: "detail-title",
							"{track.name}"
							if track.explicit {
								span { class: "explicit-badge", "E" }
							}
						}
						p { class: "detail-artists", "{artists}" }
					}

					div { class: "detail-metadata",
						div { class: "metadata-item",
							span { class: "metadata-label", "Album" }
							span { class: "metadata-value", "{track.album.name}" }
						}

						div { class: "metadata-item",
							span { class: "metadata-label", "Duration" }
							span { class: "metadata-value", "{duration}" }
						}

						div { class: "metadata-item",
							span { class: "metadata-label", "Release Date" }
							span { class: "metadata-value", "{release_date}" }
						}

						div { class: "metadata-item",
							span { class: "metadata-label", "Popularity" }
							span { class: "metadata-value",
								div { class: "popularity-bar-container",
									div {
										class: "popularity-bar",
										style: "width: {popularity}%",
									}
								}
								span { class: "popularity-text", "{popularity}%" }
							}
						}

						div { class: "metadata-item",
							span { class: "metadata-label", "Track URI" }
							span { class: "metadata-value uri", "{track_uri}" }
						}
					}

					// Spotify button
					a {
						class: "detail-spotify-button",
						href: "{spotify_url}",
						target: "_blank",
						Icon { icon: FaSpotify, width: 20, height: 20 }
						"Open in Spotify"
					}
				}
			}
		}
	}
}