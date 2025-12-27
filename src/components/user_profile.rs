use crate::models::User;
use crate::{Route, AppContext};
use dioxus::prelude::*;

#[component]
pub fn UserProfile(
    user: ReadSignal<Option<User>>,
    on_import: EventHandler<()>,
) -> Element {
    let context = use_context::<AppContext>();
    let nav = navigator();
    let mut position = use_signal(|| (0.0, 0.0));

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
								span { class: "stat-value", "{country}" }
								span { class: "stat-label", "Country" }
							}
						}
					}
				}
				div { class: "profile-actions",
					button {
						class: "import-button button",
						onclick: move |_| on_import.call(()),
						"📥 Import Playlist"
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
						"🚪 Logout"
					}
				}
			}
		} else {
			div { class: "loading", "Loading user data..." }
		}
	}
}
}