use crate::models::{RecentlyPlayedItem};
use dioxus::prelude::*;
use std::collections::BTreeMap;

#[component]
pub fn RecentlyPlayed(recent_tracks: ReadSignal<Vec<RecentlyPlayedItem>>) -> Element {
    let tracks = recent_tracks();

    // Group tracks by day
    let mut grouped: BTreeMap<String, Vec<RecentlyPlayedItem>> = BTreeMap::new();

    for item in tracks.iter() {
        // Parse the played_at timestamp and extract the date
        let date = extract_date(&item.played_at);
        grouped.entry(date).or_insert_with(Vec::new).push(item.clone());
    }

    rsx! {
		document::Link {
			rel: "stylesheet",
			href: asset!("assets/compiled/recently_played.css"),
		}
		div { class: "recently-played",
			h2 { class: "section-title", "Recently Played" }

			if grouped.is_empty() {
				div { style: "padding: 40px; text-align: center; color: #b3b3b3;",
					p { "No recently played tracks found" }
				}
			} else {
				div { class: "recently-played-groups",
					for (date , day_tracks) in grouped.iter().rev() {
						div { class: "day-group", key: "{date}",
							h3 { class: "day-header", "{format_date_header(date)}" }
							div { class: "day-tracks",
								for (index , item) in day_tracks.iter().enumerate() {
									div {
										class: "recent-track-item",
										key: "{item.played_at}-{index}",

										if let Some(image) = item.track.album.images.first() {
											img {
												class: "track-image",
												src: "{image.url}",
												alt: "{item.track.name}",
											}
										}

										div { class: "track-info",
											a {
												class: "track-name",
												href: "{item.track.external_urls.spotify}",
												target: "_blank",
												"{item.track.name}"
											}
											div { class: "track-artists",
												{item.track.artists.iter().map(|a| a.name.clone()).collect::<Vec<_>>().join(", ")}
											}
										}

										div { class: "played-time", {format_time(&item.played_at)} }
									}
								}
							}
						}
					}
				}
			}
		}
	}
}

fn extract_date(timestamp: &str) -> String {
    // Format: "2024-01-15T14:30:00Z"
    // Extract: "2024-01-15"
    timestamp.split('T').next().unwrap_or(timestamp).to_string()
}

fn format_date_header(date: &str) -> String {
    // Parse date and format nicely
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3 {
        let year = parts[0];
        let month = parts[1];
        let day = parts[2];

        let month_name = match month {
            "01" => "January",
            "02" => "February",
            "03" => "March",
            "04" => "April",
            "05" => "May",
            "06" => "June",
            "07" => "July",
            "08" => "August",
            "09" => "September",
            "10" => "October",
            "11" => "November",
            "12" => "December",
            _ => month,
        };

        // Check if it's today or yesterday
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let yesterday = (chrono::Local::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        if date == today {
            return "Today".to_string();
        } else if date == yesterday {
            return "Yesterday".to_string();
        }

        format!("{} {}, {}", month_name, day.trim_start_matches('0'), year)
    } else {
        date.to_string()
    }
}

fn format_time(timestamp: &str) -> String {
    // Format: "2024-01-15T14:30:00Z"
    // Extract: "14:30"
    if let Some(time_part) = timestamp.split('T').nth(1) {
        if let Some(time) = time_part.split(':').take(2).collect::<Vec<_>>().get(0..2) {
            return format!("{}:{}", time[0], time[1]);
        }
    }
    timestamp.to_string()
}
