use crate::components::TrackDetail;
use crate::models::Track;
use crate::utils::format_duration;
use dioxus::prelude::*;

#[component]
pub fn TopTracks(tracks: ReadSignal<Vec<Track>>) -> Element {
    let mut position = use_signal(|| (0.0, 0.0));
    let mut selected_track = use_signal(|| None::<Track>);

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
					div {
						class: "track-item clickable",
						key: "{track.id}",
						onclick: {
							let track = track.clone();
							move |_| selected_track.set(Some(track.clone()))
						},
						span { class: "track-number", "{index + 1}" }
						if let Some(image) = track.album.images.first() {
							img {
								class: "track-image",
								src: "{image.url}",
								alt: "{track.name}",
							}
						}
						div { class: "track-info",
							div { class: "track-name", "{track.name}" }
							div { class: "track-artists",
								{track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")}
							}
						}
						div { class: "track-duration", {format_duration(track.duration_ms)} }
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