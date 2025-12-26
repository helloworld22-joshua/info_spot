use crate::models::Playlist;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn Playlists(playlists: ReadSignal<Vec<Playlist>>) -> Element {
    let playlist_items = playlists();

    rsx! {
		document::Link { rel: "stylesheet", href: asset!("assets/compiled/playlists.css") }
		div { class: "playlists",
			h2 { class: "section-title", "Your Playlists" }
			if playlist_items.is_empty() {
				div { style: "padding: 40px; text-align: center; color: #b3b3b3;",
					p { "No playlists found" }
					p { style: "font-size: 0.9rem; margin-top: 10px;",
						"Create some playlists in Spotify to see them here!"
					}
				}
			} else {
				div { class: "playlists-grid",
					{
					    playlist_items
					        .iter()
					        .map(|playlist| {
					            rsx! {
						PlaylistCard { playlist: playlist.clone() }
					}
					        })
					}
				}
			}
		}
	}
}

#[component]
fn PlaylistCard(playlist: Playlist) -> Element {
    let has_image = !playlist.images.is_empty();
    let image_url = playlist.images.first().map(|i| i.url.as_str()).unwrap_or("");
    let description = playlist.description.as_deref().unwrap_or("");
    let is_public = playlist.public.unwrap_or(false);
    let visibility_text = if is_public { "Public" } else { "Private" };
    let owner_name = playlist.owner.display_name.as_deref().unwrap_or("");
    let playlist_id = playlist.id.clone();

    rsx! {
		Link {
			to: Route::PlaylistDetail {
			    id: playlist_id,
			},
			class: "playlist-card-link",
			div { class: "playlist-card",
				if has_image {
					img {
						class: "playlist-image",
						src: "{image_url}",
						alt: "{playlist.name}",
					}
				} else {
					div { class: "playlist-placeholder", "♪" }
				}
				div { class: "playlist-info",
					div { class: "playlist-name", "{playlist.name}" }
					if !description.is_empty() {
						div { class: "playlist-description", "{description}" }
					}
					div { class: "playlist-meta",
						span { class: "playlist-tracks", "{playlist.tracks.total} tracks" }
						span { class: "playlist-visibility", " • {visibility_text}" }
					}
					if !owner_name.is_empty() {
						div { class: "playlist-owner", "By {owner_name}" }
					}
				}
			}
		}
	}
}