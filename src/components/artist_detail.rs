use crate::models::Artist;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaXmark;
use dioxus_free_icons::icons::fa_brands_icons::FaSpotify;
use dioxus_free_icons::Icon;

#[component]
pub fn ArtistDetail(artist: Artist, on_close: EventHandler<()>) -> Element {
    let spotify_url = artist.external_urls.spotify.clone();
    let artist_uri = format!("spotify:artist:{}", artist.id);
    let genres = if let Some(g) = &artist.genres {
        if g.is_empty() {
            "No genres listed".to_string()
        } else {
            g.join(", ")
        }
    } else {
        "No genres listed".to_string()
    };
    let followers = if let Some(f) = &artist.followers {
        format_number(f.total)
    } else {
        "N/A".to_string()
    };
    let popularity = artist.popularity.unwrap_or(0);

    rsx! {
		document::Link { rel: "stylesheet", href: asset!("assets/compiled/detail.css") }
		div { class: "detail-overlay", onclick: move |_| on_close.call(()),

			div { class: "detail-modal", onclick: move |e| e.stop_propagation(),

				// Close button
				button {
					class: "detail-close",
					onclick: move |_| on_close.call(()),
					Icon { icon: FaXmark, width: 20, height: 20 }
				}

				// Artist image
				if let Some(images) = &artist.images {
					if let Some(image) = images.first() {
						img {
							class: "detail-cover",
							src: "{image.url}",
							alt: "{artist.name}",
						}
					}
				}

				// Artist info
				div { class: "detail-info",
					div { class: "detail-header",
						h2 { class: "detail-title", "{artist.name}" }
					}

					div { class: "detail-metadata",
						div { class: "metadata-item",
							span { class: "metadata-label", "Followers" }
							span { class: "metadata-value", "{followers}" }
						}

						div { class: "metadata-item",
							span { class: "metadata-label", "Genres" }
							span { class: "metadata-value", "{genres}" }
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
							span { class: "metadata-label", "Artist URI" }
							span { class: "metadata-value uri", "{artist_uri}" }
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

fn format_number(num: u32) -> String {
    if num >= 1_000_000 {
        format!("{:.1}M", num as f64 / 1_000_000.0)
    } else if num >= 1_000 {
        format!("{:.1}K", num as f64 / 1_000.0)
    } else {
        num.to_string()
    }
}