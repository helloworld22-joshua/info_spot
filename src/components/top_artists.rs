use crate::models::Artist;
use dioxus::prelude::*;

#[component]
pub fn TopArtists(artists: ReadSignal<Vec<Artist>>) -> Element {
    let mut position = use_signal(|| (0.0, 0.0));
    rsx! {
		document::Link {
			rel: "stylesheet",
			href: asset!("assets/compiled/top_artists.css"),
		}
		div {
			class: "top-artists component",
			onmounted: move |event| {
			    spawn(async move {
			        if let Ok(rect) = event.get_client_rect().await {
			            position.set((rect.origin.x, rect.origin.y));
			        }
			    });
			},
			style: "--position-x: {position().0}px; --position-y: {position().1}px;",
			h2 { class: "section-title", "Top Artists" }
			div { class: "artists",
				for artist in artists().iter() {
					div { class: "artist-card", key: "{artist.id}",
						if let Some(images) = &artist.images {
							if let Some(image) = images.first() {
								img {
									class: "artist-image",
									src: "{image.url}",
									alt: "{artist.name}",
								}
							}
						}
						div { class: "artist-info",
							a {
								class: "artist-name",
								href: "{artist.external_urls.spotify}",
								target: "_blank",
								"{artist.name}"
							}
							if let Some(genres) = &artist.genres {
								if !genres.is_empty() {
									div { class: "artist-genres",
										{genres.iter().take(3).cloned().collect::<Vec<_>>().join(", ")}
									}
								}
							}
							if let Some(followers) = &artist.followers {
								div { class: "artist-followers",
									"{format_number(followers.total)} Followers"
								}
							}
						}
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