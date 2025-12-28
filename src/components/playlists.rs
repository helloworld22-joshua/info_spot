use crate::models::Playlist;
use crate::{Route, AppContext};
use crate::utils::*;
use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{FaFileArrowDown, FaMagnifyingGlass};
use dioxus_free_icons::Icon;
use std::collections::HashSet;

#[component]
pub fn Playlists(playlists: ReadSignal<Vec<Playlist>>) -> Element {
    let playlist_items = playlists();
    let mut position = use_signal(|| (0.0, 0.0));
    let mut selected_playlists = use_signal(|| HashSet::<String>::new());
    let mut show_duplicates_modal = use_signal(|| false);
    let mut duplicates_data = use_signal(|| Vec::<(String, Vec<(String, String, usize)>)>::new());
    let mut removing_duplicates = use_signal(|| false);
    let context = use_context::<AppContext>();

    // Clone for use in closures
    let context_for_closures = context.clone();
    let playlist_items_for_closures = playlist_items.clone();

    // Toggle selection
    let mut toggle_selection = move |playlist_id: String| {
        let mut selected = selected_playlists.write();
        if selected.contains(&playlist_id) {
            selected.remove(&playlist_id);
        } else {
            selected.insert(playlist_id);
        }
    };

    // Select all / Deselect all
    let toggle_select_all = {
        let playlist_items = playlist_items_for_closures.clone();
        move |_| {
            let mut selected = selected_playlists.write();
            if selected.len() == playlist_items.len() {
                selected.clear();
            } else {
                selected.clear();
                for playlist in playlist_items.iter() {
                    selected.insert(playlist.id.clone());
                }
            }
        }
    };

    // Download selected playlists as ZIP
    let download_selected = {
        let context = context_for_closures.clone();
        let playlist_items = playlist_items_for_closures.clone();
        move |_| {
            let selected = selected_playlists();
            if selected.is_empty() {
                return;
            }

            let context_clone = context.clone();
            let playlist_items_clone = playlist_items.clone();

            spawn(async move {
            // Create temp directory for JSON files
            let temp_dir = std::env::temp_dir().join("spotify_playlists");
            let _ = std::fs::create_dir_all(&temp_dir);

            // Get spotify client
            let spotify_client_option = context_clone.spotify_client.read().clone();
            if let Some(client) = spotify_client_option {
                // Fetch and save each playlist
                for playlist_id in selected.iter() {
                    if let Some(playlist) = playlist_items_clone.iter().find(|p| &p.id == playlist_id) {
                        // Fetch tracks
                        match client.get_playlist_tracks(&playlist.id).await {
                            Ok(tracks) => {
                                let track_uris: Vec<String> = tracks
                                    .iter()
                                    .map(|item| &item.track)
                                    .map(|track| format!("spotify:track:{}", track.id))
                                    .collect();

                                let json_data = serde_json::json!({
                                    "info": {
                                        "name": playlist.name,
                                        "description": playlist.description.as_deref().unwrap_or(""),
                                        "author": playlist.owner.display_name.as_deref().unwrap_or("Unknown"),
                                    },
                                    "tracks": track_uris,
                                });

                                // Save to temp directory
                                let safe_name = sanitize_filename(&playlist.name);
                                let file_path = temp_dir.join(format!("{}.json", safe_name));
                                if let Ok(json_string) = serde_json::to_string_pretty(&json_data) {
                                    let _ = std::fs::write(&file_path, json_string);
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to fetch tracks for {}: {}", playlist.name, e);
                            }
                        }
                    }
                }

                // Create ZIP file
                let zip_name = "Downloaded_Playlists.zip";
                if let Some(save_path) = save_json_file(zip_name) {
                    match create_zip_from_directory(&temp_dir, &save_path) {
                        Ok(_) => {
                            show_success(&context_clone, format!("Downloaded {} playlists to ZIP", selected.len()));
                        }
                        Err(e) => {
                            show_error(&context_clone, format!("Failed to create ZIP: {}", e));
                        }
                    }
                }

                // Cleanup temp directory
                let _ = std::fs::remove_dir_all(&temp_dir);
            }
        });
        }
    };

    // Find duplicates in selected playlists
    let find_duplicates = {
        let context = context_for_closures.clone();
        let playlist_items = playlist_items_for_closures.clone();
        move |_| {
            let selected = selected_playlists();
            if selected.is_empty() {
                return;
            }

            let spotify_client_option = context.spotify_client.read().clone();
            let playlist_items_clone = playlist_items.clone();
            let context_clone = context.clone();

            if let Some(client) = spotify_client_option {
            spawn(async move {
                let mut all_duplicates = Vec::new();

                for playlist_id in selected.iter() {
                    if let Some(playlist) = playlist_items_clone.iter().find(|p| &p.id == playlist_id) {
                        match client.get_playlist_tracks(&playlist.id).await {
                            Ok(tracks) => {
                                // Find duplicates in this playlist
                                let mut track_map = std::collections::HashMap::new();
                                for (idx, item) in tracks.iter().enumerate() {
                                    let track = &item.track;
                                    track_map
                                        .entry(track.id.clone())
                                        .or_insert_with(Vec::new)
                                        .push(idx);
                                }

                                let playlist_duplicates: Vec<(String, String, usize)> = track_map
                                    .into_iter()
                                    .filter(|(_, indices)| indices.len() > 1)
                                    .map(|(track_id, indices)| {
                                        let track = tracks
                                            .iter()
                                            .find_map(|item| {
                                                Some(&item.track).filter(|t| t.id == track_id)
                                            })
                                            .unwrap();
                                        (
                                            format!("{} - {}", track.name, track.artists.first().map(|a| a.name.as_str()).unwrap_or("Unknown")),
                                            track_id,
                                            indices.len(),
                                        )
                                    })
                                    .collect();

                                if !playlist_duplicates.is_empty() {
                                    all_duplicates.push((playlist.name.clone(), playlist_duplicates));
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to check duplicates for {}: {}", playlist.name, e);
                            }
                        }
                    }
                }

                duplicates_data.set(all_duplicates.clone());
                if all_duplicates.is_empty() {
                    show_info(&context_clone, "No duplicates found in selected playlists".to_string());
                } else {
                    show_duplicates_modal.set(true);
                }
            });
        }
        }
    };

    // Remove duplicates from selected playlists
    let remove_duplicates = {
        let context = context_for_closures.clone();
        let playlist_items = playlist_items_for_closures.clone();
        move |_| {
            removing_duplicates.set(true);
            let selected = selected_playlists();

            let spotify_client_option = context.spotify_client.read().clone();
            let playlist_items_clone = playlist_items.clone();
            let context_clone = context.clone();

            if let Some(client) = spotify_client_option {
            spawn(async move {
                let mut total_removed = 0;

                for playlist_id in selected.iter() {
                    if let Some(playlist) = playlist_items_clone.iter().find(|p| &p.id == playlist_id) {
                        match client.get_playlist_tracks(&playlist.id).await {
                            Ok(tracks) => {
                                // Find duplicate positions
                                let mut track_map = std::collections::HashMap::new();
                                for (idx, item) in tracks.iter().enumerate() {
                                    let track = &item.track;
                                    track_map
                                        .entry(track.id.clone())
                                        .or_insert_with(Vec::new)
                                        .push(idx);
                                }

                                // Get tracks with URIs and positions to remove (keep first occurrence)
                                let tracks_to_remove: Vec<(String, usize)> = track_map
                                    .iter()
                                    .filter(|(_, indices)| indices.len() > 1)
                                    .flat_map(|(track_id, indices)| {
                                        let uri = format!("spotify:track:{}", track_id);
                                        indices.iter().skip(1).map(move |&pos| (uri.clone(), pos))
                                    })
                                    .collect();

                                if !tracks_to_remove.is_empty() {
                                    match client.remove_tracks_from_playlist(&playlist.id, tracks_to_remove).await {
                                        Ok(_) => {
                                            total_removed += 1;
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to remove duplicates from {}: {}", playlist.name, e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to process {}: {}", playlist.name, e);
                            }
                        }
                    }
                }

                removing_duplicates.set(false);
                show_duplicates_modal.set(false);

                if total_removed > 0 {
                    show_success(&context_clone, format!("Removed duplicates from {} playlist(s)", total_removed));
                }
            });
        }
        }
    };

    let selected_count = selected_playlists().len();
    let all_selected = selected_count == playlist_items.len() && !playlist_items.is_empty();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/compiled/playlists.css") }
        div {
            class: "playlists component",
            onmounted: move |event| {
                spawn(async move {
                    if let Ok(rect) = event.get_client_rect().await {
                        position.set((rect.origin.x, rect.origin.y));
                    }
                });
            },
            style: "--position-x: {position().0}px; --position-y: {position().1}px;",

            div { class: "playlists-header",
                h2 { class: "section-title", "Your Playlists" }

                if !playlist_items.is_empty() {
                    div { class: "playlists-actions",
                        button {
                            class: "select-all-button",
                            onclick: toggle_select_all,
                            if all_selected { "✓ Deselect All" } else { "Select All" }
                        }

                        if selected_count > 0 {
                            span { class: "selected-count", "{selected_count} selected" }

                            button {
                                class: "batch-download-button",
                                onclick: download_selected,
                                Icon {
                                    icon: FaFileArrowDown,
                                    width: 16,
                                    height: 16,
                                }
                                "Download ({selected_count})"
                            }

                            button {
                                class: "batch-duplicates-button",
                                onclick: find_duplicates,
                                Icon {
                                    icon: FaMagnifyingGlass,
                                    width: 16,
                                    height: 16,
                                }
                                "Find Duplicates"
                            }
                        }
                    }
                }
            }

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
                                let playlist_id = playlist.id.clone();
                                let is_selected = selected_playlists().contains(&playlist_id);
                                rsx! {
                                    PlaylistCard {
                                        playlist: playlist.clone(),
                                        is_selected: is_selected,
                                        on_toggle: move |_| toggle_selection(playlist_id.clone()),
                                    }
                                }
                            })
                    }
                }
            }
        }

        // Duplicates Modal
        if show_duplicates_modal() {
            div { class: "modal-overlay",
                onclick: move |_| show_duplicates_modal.set(false),
                div {
                    class: "modal-content duplicates-modal",
                    onclick: move |e| e.stop_propagation(),

                    div { class: "modal-header",
                        h2 { "Duplicate Tracks Found" }
                        p { class: "modal-subtitle",
                            "Found duplicates in {duplicates_data().len()} playlist(s)"
                        }
                    }

                    div { class: "duplicates-list",
                        for (playlist_name, dups) in duplicates_data().iter() {
                            div { class: "playlist-duplicates-section",
                                h3 { class: "playlist-name-header", "{playlist_name}" }
                                for (track_info, _track_id, count) in dups.iter() {
                                    div { class: "duplicate-item",
                                        div { class: "duplicate-track-info",
                                            strong { "{track_info}" }
                                            div { class: "duplicate-count",
                                                "Found {count} times"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "modal-actions",
                        button {
                            class: "modal-button cancel-button",
                            onclick: move |_| show_duplicates_modal.set(false),
                            "Close"
                        }
                        button {
                            class: "modal-button remove-button",
                            onclick: remove_duplicates,
                            disabled: removing_duplicates(),
                            if removing_duplicates() {
                                "Removing..."
                            } else {
                                "Remove All Duplicates"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PlaylistCard(
    playlist: Playlist,
    is_selected: bool,
    on_toggle: EventHandler<()>,
) -> Element {
    let has_image = !playlist.images.is_empty();
    let image_url = playlist.images.first().map(|i| i.url.as_str()).unwrap_or("");
    let description = playlist.description.as_deref().unwrap_or("");
    let is_public = playlist.public.unwrap_or(false);
    let visibility_text = if is_public { "Public" } else { "Private" };
    let owner_name = playlist.owner.display_name.as_deref().unwrap_or("");
    let playlist_id = playlist.id.clone();

    rsx! {
        div {
            class: if is_selected { "playlist-card-wrapper selected" } else { "playlist-card-wrapper" },

            div {
                class: "playlist-checkbox",
                onclick: move |e| {
                    e.stop_propagation();
                    on_toggle.call(());
                },
                input {
                    r#type: "checkbox",
                    checked: is_selected,
                    onchange: move |_| {},
                }
            }

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
}