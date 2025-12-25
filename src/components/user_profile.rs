use crate::models::User;
use dioxus::prelude::*;

#[component]
pub fn UserProfile(user: ReadOnlySignal<Option<User>>) -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/compiled/user_profile.css") }
        div { class: "user-profile",
            if let Some(user_data) = user() {
                div { class: "profile-card",
                    if let Some(image) = user_data.images.first() {
                        img {
                            class: "profile-image",
                            src: "{image.url}",
                            alt: "Profile"
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
                }
            } else {
                div { class: "loading", "Loading user data..." }
            }
        }
    }
}
