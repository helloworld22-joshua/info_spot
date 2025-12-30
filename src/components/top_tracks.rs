use crate::components::TrackDetail;
use crate::models::Track;
use crate::utils::format_duration;
use dioxus::prelude::*;

#[component]
pub fn TopTracks(tracks: ReadSignal<Vec<Track>>) -> Element {
    let mut position = use_signal(|| (0.0, 0.0));
    let mut selected_track = use_signal(|| None::<Track>);

    rsx! {
		document::Link { rel: "stylesheet", href: asset!("assets/compiled/top.css") }
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
			div { class: "top-scroll-container",
				for (index , track) in tracks().iter().enumerate() {
					div {
						class: "top-card",
						key: "{track.id}",
						onclick: {
						    let track = track.clone();
						    move |_| selected_track.set(Some(track.clone()))
						},

						// Rank badge
						span { class: "top-rank", "#{index + 1}" }

						// Album cover
						if let Some(image) = track.album.images.first() {
							img {
								class: "top-image track",
								src: "{image.url}",
								alt: "{track.name}",
							}
						}

						// Track info
						div { class: "top-info",
							div { class: "top-name", "{track.name}" }
							div { class: "top-artist",
								{track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")}
							}
							div { class: "top-album", "{track.album.name}" }
							div { class: "top-duration", {format_duration(track.duration_ms)} }
						}
					}
				}
			}
		}

		// Track detail modal
		if let Some(track) = selected_track() {
			TrackDetail { track, on_close: move |_| selected_track.set(None) }
		}
	}
}