use crate::models::Track;
use dioxus::prelude::*;

#[component]
pub fn TopTracks(tracks: ReadSignal<Vec<Track>>) -> Element {
    let mut position = use_signal(|| (0.0, 0.0));
    rsx! {
		document::Link { rel: "stylesheet", href: asset!("assets/compiled/top_tracks.css") }
		div {
			class: "top-tracks component",
			onmounted: move |event| {
			    spawn(async move {
			        if let Ok(rect) = event.get_client_rect().await {
			            position.set((rect.origin.x, rect.origin.y));
			        }
			    });
			},
			style: "--position-x: {position().0}px; --position-y: {position().1}px;",
			h2 { class: "section-title", "Top Tracks" }
			div { class: "tracks-list",
				for (index , track) in tracks().iter().enumerate() {
					div { class: "track-item", key: "{track.id}",
						span { class: "track-number", "{index + 1}" }
						if let Some(image) = track.album.images.first() {
							img {
								class: "track-image",
								src: "{image.url}",
								alt: "{track.name}",
							}
						}
						div { class: "track-info",
							a {
								class: "track-name",
								href: "{track.external_urls.spotify}",
								target: "_blank",
								"{track.name}"
							}
							div { class: "track-artists",
								{track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")}
							}
						}
						div { class: "track-duration", {format_duration(track.duration_ms)} }
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