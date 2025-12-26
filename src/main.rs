mod api;
mod components;
mod models;
mod oauth;

use crate::api::SpotifyClient;
use crate::components::{UserProfile, TopTracks, TopArtists, Playlists, RecentlyPlayed};
use crate::models::*;
use dioxus::prelude::*;
use std::rc::Rc;

// Global context for sharing Spotify client across routes
#[derive(Clone)]
struct AppContext {
    spotify_client: Signal<Option<Rc<SpotifyClient>>>,
}

fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Initialize global Spotify client context
    use_context_provider(|| AppContext {
        spotify_client: Signal::new(None),
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/compiled/main.css") }
        Router::<Route> {}
    }
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/callback")]
    Callback {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/playlist/:id")]
    PlaylistDetail { id: String },
}

#[component]
fn Home() -> Element {
    let nav = navigator();
    let mut context = use_context::<AppContext>();

    let client_id = std::env::var("SPOTIFY_CLIENT_ID")
        .unwrap_or_else(|_| "".to_string());
    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET")
        .unwrap_or_else(|_| "".to_string());
    let redirect_uri = std::env::var("SPOTIFY_REDIRECT_URI")
        .unwrap_or_else(|_| "http://127.0.0.1:8888/callback".to_string());

    // Check if credentials are configured
    let credentials_missing = client_id.is_empty() || client_secret.is_empty();

    if credentials_missing {
        eprintln!("ERROR: Spotify credentials not found!");
        eprintln!("Please create a .env file with:");
        eprintln!("  SPOTIFY_CLIENT_ID=your_client_id");
        eprintln!("  SPOTIFY_CLIENT_SECRET=your_client_secret");
        eprintln!("  SPOTIFY_REDIRECT_URI=http://127.0.0.1:8888/callback");
    }

    let mut error_msg = use_signal(|| None::<String>);
    let mut authenticating = use_signal(|| false);

    let handle_login = move |_| {
        // Check credentials before attempting login
        if client_id.is_empty() || client_secret.is_empty() {
            error_msg.set(Some(
                "Missing Spotify credentials! Please check your .env file.".to_string()
            ));
            return;
        }

        authenticating.set(true);
        error_msg.set(None);

        let client_id_clone = client_id.clone();
        let client_secret_clone = client_secret.clone();
        let redirect_uri_clone = redirect_uri.clone();
        let nav_clone = nav.clone();

        spawn(async move {
            let spotify_client = SpotifyClient::new(
                client_id_clone,
                client_secret_clone,
                redirect_uri_clone
            );

            let state = generate_random_string(16);
            let auth_url = spotify_client.get_auth_url(&state);

            // Open browser
            if let Err(e) = open::that(&auth_url) {
                eprintln!("Failed to open browser: {}", e);
                error_msg.set(Some(format!("Failed to open browser: {}", e)));
                authenticating.set(false);
                return;
            }

            // Wait for callback
            match oauth::start_callback_server() {
                Ok(code) => {
                    // Exchange code for token
                    match spotify_client.exchange_code(&code).await {
                        Ok(token_response) => {
                            // Store token
                            spotify_client.set_token(token_response.access_token).await;

                            // Store client in context
                            context.spotify_client.set(Some(Rc::new(spotify_client)));

                            // Navigate to dashboard
                            nav_clone.push(Route::Dashboard {});
                        }
                        Err(e) => {
                            error_msg.set(Some(format!("Authentication failed: {}", e)));
                            authenticating.set(false);
                        }
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("Callback error: {}", e)));
                    authenticating.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "home-container",
            div { class: "login-card",
                h1 { class: "app-title", "InfoSpot" }
                p { class: "app-description",
                    "Connect with Spotify to view your music stats and playlists"
                }

                if credentials_missing {
                    div { style: "background: #ff4444; padding: 15px; border-radius: 8px; margin-bottom: 20px;",
                        p { style: "margin-bottom: 10px; font-weight: bold;", "⚠️ Missing Spotify Credentials" }
                        p { style: "font-size: 0.9rem; margin-bottom: 5px;",
                            "Please create a .env file in the project root with:"
                        }
                        pre { style: "background: rgba(0,0,0,0.3); padding: 10px; border-radius: 4px; font-size: 0.85rem; overflow-x: auto;",
                            "SPOTIFY_CLIENT_ID=your_client_id_here\n"
                            "SPOTIFY_CLIENT_SECRET=your_client_secret_here\n"
                            "SPOTIFY_REDIRECT_URI=http://127.0.0.1:8888/callback"
                        }
                        p { style: "font-size: 0.85rem; margin-top: 10px;",
                            "Get your credentials at: "
                            a {
                                href: "https://developer.spotify.com/dashboard",
                                target: "_blank",
                                style: "color: #1ed760; text-decoration: underline;",
                                "Spotify Developer Dashboard"
                            }
                        }
                    }
                }

                if authenticating() {
                    p { class: "loading", "Authenticating..." }
                } else {
                    button {
                        class: "login-button",
                        onclick: handle_login,
                        disabled: credentials_missing,
                        style: if credentials_missing { "opacity: 0.5; cursor: not-allowed;" } else { "" },
                        "Login with Spotify"
                    }
                }

                if let Some(error) = error_msg() {
                    p { style: "color: #ff4444; margin-top: 20px;", "{error}" }
                }
            }
        }
    }
}

#[component]
fn Callback() -> Element {
    let nav = navigator();

    // Note: In a real implementation, you'd need to handle the OAuth callback
    // This is a simplified version - you'll need to implement a local server
    // to capture the authorization code from the redirect

    use_effect(move || {
        spawn(async move {
            // This is where you'd extract the code from the URL and exchange it
            // For now, this is a placeholder
            nav.push(Route::Dashboard {});
        });
    });

    rsx! {
        div { class: "callback-container",
            p { "Processing authentication..." }
        }
    }
}

#[component]
fn Dashboard() -> Element {
    let context = use_context::<AppContext>();
    let nav = navigator();

    let mut user = use_signal(|| None::<User>);
    let mut top_tracks = use_signal(|| Vec::<Track>::new());
    let mut top_artists = use_signal(|| Vec::<Artist>::new());
    let mut playlists = use_signal(|| Vec::<Playlist>::new());
    let mut recently_played = use_signal(|| Vec::<RecentlyPlayedItem>::new());
    let mut time_range = use_signal(|| "short_term".to_string());
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);

    // Check if we have a Spotify client with token
    let spotify_client_option = context.spotify_client.read().clone();

    if spotify_client_option.is_none() {
        // No authenticated client, redirect to home
        use_effect(move || {
            nav.push(Route::Home {});
        });

        return rsx! {
            div { class: "loading", "Redirecting to login..." }
        };
    }

    let client = spotify_client_option.unwrap();

    // Initial data fetch
    {
        let client_clone = client.clone();
        use_effect(move || {
            let client_clone2 = client_clone.clone();
            let current_time_range = time_range();

            spawn(async move {
                // Fetch user data
                match client_clone2.get_current_user().await {
                    Ok(user_data) => {
                        user.set(Some(user_data));
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch user: {}", e);
                        error.set(Some(format!("Failed to fetch user data: {}", e)));
                    }
                }

                // Fetch top tracks
                match client_clone2.get_top_tracks(20, &current_time_range).await {
                    Ok(tracks) => {
                        top_tracks.set(tracks);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch top tracks: {}", e);
                    }
                }

                // Fetch top artists
                match client_clone2.get_top_artists(20, &current_time_range).await {
                    Ok(artists) => {
                        top_artists.set(artists);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch top artists: {}", e);
                    }
                }

                // Fetch playlists
                match client_clone2.get_playlists(50).await {
                    Ok(user_playlists) => {
                        println!("DEBUG: Fetched {} playlists", user_playlists.len());
                        playlists.set(user_playlists);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch playlists: {}", e);
                    }
                }

                // Fetch recently played (last 50 tracks)
                match client_clone2.get_recently_played(50).await {
                    Ok(recent_tracks) => {
                        println!("DEBUG: Fetched {} recently played tracks", recent_tracks.len());
                        recently_played.set(recent_tracks);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch recently played: {}", e);
                    }
                }

                loading.set(false);
            });
        });
    }

    // Create event handlers for time range buttons
    let client_for_short = client.clone();
    let on_short_term = move |_| {
        let new_range = "short_term".to_string();
        time_range.set(new_range.clone());
        loading.set(true);

        let client_clone = client_for_short.clone();
        spawn(async move {
            if let Ok(tracks) = client_clone.get_top_tracks(20, &new_range).await {
                top_tracks.set(tracks);
            }

            if let Ok(artists) = client_clone.get_top_artists(20, &new_range).await {
                top_artists.set(artists);
            }

            loading.set(false);
        });
    };

    let client_for_medium = client.clone();
    let on_medium_term = move |_| {
        let new_range = "medium_term".to_string();
        time_range.set(new_range.clone());
        loading.set(true);

        let client_clone = client_for_medium.clone();
        spawn(async move {
            if let Ok(tracks) = client_clone.get_top_tracks(20, &new_range).await {
                top_tracks.set(tracks);
            }

            if let Ok(artists) = client_clone.get_top_artists(20, &new_range).await {
                top_artists.set(artists);
            }

            loading.set(false);
        });
    };

    let client_for_long = client.clone();
    let on_long_term = move |_| {
        let new_range = "long_term".to_string();
        time_range.set(new_range.clone());
        loading.set(true);

        let client_clone = client_for_long.clone();
        spawn(async move {
            if let Ok(tracks) = client_clone.get_top_tracks(20, &new_range).await {
                top_tracks.set(tracks);
            }

            if let Ok(artists) = client_clone.get_top_artists(20, &new_range).await {
                top_artists.set(artists);
            }

            loading.set(false);
        });
    };

    // Import playlist handler
    let on_import_playlist = {
        let client_for_import = client.clone();
        move |_| {
            let client_clone = client_for_import.clone();
            spawn(async move {
                if let Some(file_path) = pick_json_file() {
                    import_playlist(client_clone, file_path).await;
                }
            });
        }
    };

    let mut mouse_pos = use_signal(|| (0, 0));
    rsx! {
        div { class: "dashboard-container",
            onmousemove: move |event| {
                let coords = event.data().client_coordinates();
                mouse_pos.set((coords.x as i32, coords.y as i32));
            },
            style: "--mouse-x: {mouse_pos().0}px; --mouse-y: {mouse_pos().1}px;",
            header { class: "dashboard-header",
                div { class: "header-row",
                    h1 { class: "dashboard-title", "Your Spotify Stats" }
                }
                div { class: "time-range-selector",
                    button {
                        class: if time_range() == "short_term" { "active" } else { "" },
                        onclick: on_short_term,
                        "Last 4 Weeks"
                    }
                    button {
                        class: if time_range() == "medium_term" { "active" } else { "" },
                        onclick: on_medium_term,
                        "Last 6 Months"
                    }
                    button {
                        class: if time_range() == "long_term" { "active" } else { "" },
                        onclick: on_long_term,
                        "All Time"
                    }
                }
            }

            if let Some(err) = error() {
                div { style: "padding: 20px; background: rgba(255,68,68,0.1); border: 1px solid #ff4444; border-radius: 8px; margin: 20px;",
                    p { style: "color: #ff4444; margin: 0;", "Error: {err}" }
                    p { style: "color: #b3b3b3; margin-top: 10px; font-size: 0.9rem;",
                        "Please try logging in again or check the console for details."
                    }
                }
            }

            if loading() {
                div { class: "loading", "Loading your data..." }
            } else {
                div { class: "dashboard-content",
                    UserProfile { user: user, on_import: on_import_playlist }
                    TopTracks { tracks: top_tracks }
                    TopArtists { artists: top_artists }
                    Playlists { playlists: playlists }
                    RecentlyPlayed { recent_tracks: recently_played }
                }
            }
        }
    }
}

#[component]
fn PlaylistDetail(id: String) -> Element {
    let context = use_context::<AppContext>();
    let nav = navigator();

    let mut tracks = use_signal(|| Vec::<PlaylistTrackItem>::new());
    let mut playlist_info = use_signal(|| None::<Playlist>);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut sort_order = use_signal(|| "default".to_string());

    // Check if we have a Spotify client with token
    let spotify_client_option = context.spotify_client.read().clone();

    if spotify_client_option.is_none() {
        use_effect(move || {
            nav.push(Route::Home {});
        });

        return rsx! {
            div { class: "loading", "Redirecting to login..." }
        };
    }

    let client = spotify_client_option.unwrap();

    // Fetch playlist info and tracks
    {
        let client_clone = client.clone();
        let playlist_id = id.clone();
        use_effect(move || {
            let client_clone2 = client_clone.clone();
            let playlist_id_clone = playlist_id.clone();

            spawn(async move {
                // Fetch playlist details
                match client_clone2.get_playlist(&playlist_id_clone).await {
                    Ok(playlist) => {
                        playlist_info.set(Some(playlist));
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch playlist info: {}", e);
                    }
                }

                // Fetch playlist tracks
                match client_clone2.get_playlist_tracks(&playlist_id_clone).await {
                    Ok(playlist_tracks) => {
                        println!("DEBUG: Fetched {} tracks", playlist_tracks.len());
                        tracks.set(playlist_tracks);
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch playlist tracks: {}", e);
                        error.set(Some(format!("Failed to load tracks: {}", e)));
                    }
                }
                loading.set(false);
            });
        });
    }

    // Download playlist as JSON
    let download_json = move |_| {
        let playlist = playlist_info();
        let tracks_list = tracks();

        if let Some(pl) = playlist {
            spawn(async move {
                // Create JSON structure
                let json_data = serde_json::json!({
                    "info": {
                        "name": pl.name,
                        "id": pl.id,
                        "author": pl.owner.display_name.unwrap_or_else(|| "Unknown".to_string()),
                        "description": pl.description.unwrap_or_else(|| "".to_string()),
                    },
                    "tracks": tracks_list.iter().map(|item| {
                        format!("spotify:track:{}", item.track.id)
                    }).collect::<Vec<_>>()
                });

                // Convert to pretty JSON string
                match serde_json::to_string_pretty(&json_data) {
                    Ok(json_string) => {
                        // Sanitize filename
                        let filename = sanitize_filename(&pl.name);
                        let filepath = format!("{}.json", filename);

                        // Write to file in Downloads or home directory
                        let home_dir = std::env::var("HOME")
                            .or_else(|_| std::env::var("USERPROFILE"))
                            .unwrap_or_else(|_| ".".to_string());

                        let downloads_path = format!("{}/Downloads/{}", home_dir, filepath);
                        let home_path = format!("{}/{}", home_dir, filepath);

                        // Try Downloads first, fall back to home directory
                        let result = std::fs::write(&downloads_path, &json_string)
                            .or_else(|_| std::fs::write(&home_path, &json_string));

                        match result {
                            Ok(_) => {
                                println!("✓ Playlist exported to: {}",
                                    if std::path::Path::new(&downloads_path).exists() {
                                        downloads_path
                                    } else {
                                        home_path
                                    }
                                );
                            }
                            Err(e) => {
                                eprintln!("Failed to save playlist: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to serialize JSON: {}", e);
                    }
                }
            });
        }
    };

    // Sort tracks based on selected order
    let sorted_tracks = {
        let mut tracks_vec = tracks();
        match sort_order().as_str() {
            "name_asc" => tracks_vec.sort_by(|a, b| a.track.name.to_lowercase().cmp(&b.track.name.to_lowercase())),
            "name_desc" => tracks_vec.sort_by(|a, b| b.track.name.to_lowercase().cmp(&a.track.name.to_lowercase())),
            "artist_asc" => tracks_vec.sort_by(|a, b| {
                let a_artist = a.track.artists.first().map(|ar| ar.name.to_lowercase()).unwrap_or_default();
                let b_artist = b.track.artists.first().map(|ar| ar.name.to_lowercase()).unwrap_or_default();
                a_artist.cmp(&b_artist)
            }),
            "artist_desc" => tracks_vec.sort_by(|a, b| {
                let a_artist = a.track.artists.first().map(|ar| ar.name.to_lowercase()).unwrap_or_default();
                let b_artist = b.track.artists.first().map(|ar| ar.name.to_lowercase()).unwrap_or_default();
                b_artist.cmp(&a_artist)
            }),
            "date_added_desc" => tracks_vec.sort_by(|a, b| b.added_at.cmp(&a.added_at)),
            "date_added_asc" => tracks_vec.sort_by(|a, b| a.added_at.cmp(&b.added_at)),
            _ => {} // default - keep original order
        }
        tracks_vec
    };

    rsx! {
        div { class: "playlist-detail-container",
            header { class: "playlist-detail-header",
                div { class: "header-actions",
                    button {
                        class: "back-button",
                        onclick: move |_| {
                            nav.push(Route::Dashboard {});
                        },
                        "← Back to Dashboard"
                    }
                    if playlist_info().is_some() {
                        button {
                            class: "download-button",
                            onclick: download_json,
                            "⬇ Download JSON"
                        }
                    }
                }
                h1 { class: "playlist-detail-title",
                    {playlist_info().as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "Playlist Tracks".to_string())}
                }
            }

            div { class: "sort-controls",
                span { class: "sort-label", "Sort by:" }
                button {
                    class: if sort_order() == "default" { "sort-button active" } else { "sort-button" },
                    onclick: move |_| sort_order.set("default".to_string()),
                    "Default Order"
                }
                button {
                    class: if sort_order() == "name_asc" { "sort-button active" } else { "sort-button" },
                    onclick: move |_| sort_order.set("name_asc".to_string()),
                    "Name A-Z"
                }
                button {
                    class: if sort_order() == "name_desc" { "sort-button active" } else { "sort-button" },
                    onclick: move |_| sort_order.set("name_desc".to_string()),
                    "Name Z-A"
                }
                button {
                    class: if sort_order() == "artist_asc" { "sort-button active" } else { "sort-button" },
                    onclick: move |_| sort_order.set("artist_asc".to_string()),
                    "Artist A-Z"
                }
                button {
                    class: if sort_order() == "artist_desc" { "sort-button active" } else { "sort-button" },
                    onclick: move |_| sort_order.set("artist_desc".to_string()),
                    "Artist Z-A"
                }
                button {
                    class: if sort_order() == "date_added_desc" { "sort-button active" } else { "sort-button" },
                    onclick: move |_| sort_order.set("date_added_desc".to_string()),
                    "Newest First"
                }
                button {
                    class: if sort_order() == "date_added_asc" { "sort-button active" } else { "sort-button" },
                    onclick: move |_| sort_order.set("date_added_asc".to_string()),
                    "Oldest First"
                }
            }

            if let Some(err) = error() {
                div { class: "error-message",
                    p { "Error: {err}" }
                }
            }

            if loading() {
                div { class: "loading", "Loading tracks..." }
            } else {
                div { class: "tracks-list",
                    for (index, item) in sorted_tracks.iter().enumerate() {
                        div {
                            class: "track-item",
                            key: "{item.track.id}-{index}",
                            span { class: "track-number", "{index + 1}" }
                            if let Some(image) = item.track.album.images.first() {
                                img {
                                    class: "track-image",
                                    src: "{image.url}",
                                    alt: "{item.track.name}"
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
                                    {item.track.artists.iter()
                                        .map(|a| a.name.clone())
                                        .collect::<Vec<_>>()
                                        .join(", ")}
                                }
                            }
                            div { class: "track-duration",
                                {format_duration(item.track.duration_ms)}
                            }
                        }
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

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

fn pick_json_file() -> Option<String> {
    use std::process::Command;

    // Use native file picker based on OS
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(r#"POSIX path of (choose file with prompt "Select a playlist JSON file" of type {"public.json"})"#)
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            let trimmed = path.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = Command::new("zenity")
            .args(&["--file-selection", "--file-filter=*.json"])
            .output()
            .ok()?;

        if output.status.success() {
            let path = String::from_utf8(output.stdout).ok()?;
            return Some(path.trim().to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // For Windows, we'll use a simple dialog
        println!("Please enter the full path to the JSON file:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok()?;
        return Some(input.trim().to_string());
    }

    None
}

async fn import_playlist(client: Rc<SpotifyClient>, file_path: String) {
    println!("Importing playlist from: {}", file_path);

    // Read the JSON file
    let file_content = match std::fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            return;
        }
    };

    // Parse JSON
    let json_data: serde_json::Value = match serde_json::from_str(&file_content) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            return;
        }
    };

    // Extract playlist info
    let name = json_data["info"]["name"].as_str().unwrap_or("Imported Playlist");
    let description = json_data["info"]["description"].as_str().unwrap_or("");

    // Extract track URIs
    let tracks: Vec<String> = json_data["tracks"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|t| t.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if tracks.is_empty() {
        eprintln!("No tracks found in JSON file");
        return;
    }

    println!("Creating playlist '{}' with {} tracks...", name, tracks.len());

    // Create the playlist
    match client.create_playlist(name, description, false).await {
        Ok(playlist) => {
            println!("✓ Created playlist: {}", playlist.name);

            // Add tracks to the playlist
            match client.add_tracks_to_playlist(&playlist.id, tracks).await {
                Ok(_) => {
                    println!("✓ Successfully imported playlist!");
                    println!("  View it in Spotify: {}", playlist.external_urls.spotify);
                }
                Err(e) => {
                    eprintln!("Failed to add tracks to playlist: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to create playlist: {}", e);
        }
    }
}

fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}