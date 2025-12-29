use crate::components::ArtistDetail;
use crate::models::Artist;
use dioxus::prelude::*;

#[component]
pub fn TopArtists(artists: ReadSignal<Vec<Artist>>) -> Element {
    let mut position = use_signal(|| (0.0, 0.0));
    let mut selected_artist = use_signal(|| None::<Artist>);
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
					div {
						class: "artist-card clickable",
						key: "{artist.id}",
						onclick: {
							let artist = artist.clone();
							move |_| selected_artist.set(Some(artist.clone()))
						},
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
							div { class: "artist-name", "{artist.name}" }
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

		// Artist detail modal
		if let Some(artist) = selected_artist() {
			ArtistDetail {
				artist: artist,
				on_close: move |_| selected_artist.set(None),
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