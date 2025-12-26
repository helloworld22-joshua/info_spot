use crate::models::*;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: String,
}

#[derive(Clone)]
pub struct SpotifyClient {
    client: Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    token: Arc<RwLock<Option<String>>>,
}

impl SpotifyClient {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client: Client::new(),
            client_id,
            client_secret,
            redirect_uri,
            token: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_auth_url(&self, state: &str) -> String {
        let scopes = vec![
            "user-read-private",
            "user-read-email",
            "user-top-read",
            "user-read-recently-played",
            "playlist-read-private",
            "playlist-read-collaborative",
            "playlist-modify-public",
            "playlist-modify-private",
        ]
        .join(" ");

        format!(
            "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}",
            self.client_id,
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
            state
        )
    }

    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        let auth_header = general_purpose::STANDARD.encode(format!(
            "{}:{}",
            self.client_id, self.client_secret
        ));

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
        ];

        let response = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", auth_header))
            .form(&params)
            .send()
            .await
            .context("Failed to exchange code")?;

        let token_response: TokenResponse = response
            .json()
            .await
            .context("Failed to parse token response")?;

        *self.token.write().await = Some(token_response.access_token.clone());

        Ok(token_response)
    }

    pub async fn set_token(&self, token: String) {
        *self.token.write().await = Some(token);
    }

    async fn get_token(&self) -> Option<String> {
        self.token.read().await.clone()
    }

    pub async fn get_current_user(&self) -> Result<User> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let response = self
            .client
            .get("https://api.spotify.com/v1/me")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch user")?;

        response.json().await.context("Failed to parse user")
    }

    pub async fn get_top_tracks(&self, limit: u32, time_range: &str) -> Result<Vec<Track>> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let url = format!(
            "https://api.spotify.com/v1/me/top/tracks?limit={}&time_range={}",
            limit, time_range
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch top tracks")?;

        let tracks_response: TopTracksResponse = response
            .json()
            .await
            .context("Failed to parse top tracks")?;

        Ok(tracks_response.items)
    }

    pub async fn get_top_artists(&self, limit: u32, time_range: &str) -> Result<Vec<Artist>> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let url = format!(
            "https://api.spotify.com/v1/me/top/artists?limit={}&time_range={}",
            limit, time_range
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch top artists")?;

        let artists_response: TopArtistsResponse = response
            .json()
            .await
            .context("Failed to parse top artists")?;

        Ok(artists_response.items)
    }

    pub async fn get_playlists(&self, limit: u32) -> Result<Vec<Playlist>> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let url = format!("https://api.spotify.com/v1/me/playlists?limit={}", limit);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch playlists")?;

        let status = response.status();
        let response_text = response.text().await?;

        println!("DEBUG Playlists - Status: {}", status);
        println!("DEBUG Playlists - Response: {}", response_text);

        let playlists_response: PlaylistsResponse = serde_json::from_str(&response_text)
            .context(format!("Failed to parse playlists response: {}", response_text))?;

        Ok(playlists_response.items)
    }

    pub async fn get_playlist(&self, playlist_id: &str) -> Result<Playlist> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let url = format!("https://api.spotify.com/v1/playlists/{}", playlist_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch playlist")?;

        let playlist: Playlist = response
            .json()
            .await
            .context("Failed to parse playlist")?;

        Ok(playlist)
    }

    pub async fn create_playlist(&self, name: &str, description: &str, public: bool) -> Result<Playlist> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        // Get current user ID first
        let user = self.get_current_user().await?;

        let url = format!("https://api.spotify.com/v1/users/{}/playlists", user.id);

        let body = serde_json::json!({
            "name": name,
            "description": description,
            "public": public
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to create playlist")?;

        let playlist: Playlist = response
            .json()
            .await
            .context("Failed to parse created playlist")?;

        Ok(playlist)
    }

    pub async fn add_tracks_to_playlist(&self, playlist_id: &str, track_uris: Vec<String>) -> Result<()> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let url = format!("https://api.spotify.com/v1/playlists/{}/tracks", playlist_id);

        // Spotify API limits to 100 tracks per request
        for chunk in track_uris.chunks(100) {
            let body = serde_json::json!({
                "uris": chunk
            });

            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .context("Failed to add tracks to playlist")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await?;
                return Err(anyhow::anyhow!("Failed to add tracks: {} - {}", status, text));
            }
        }

        Ok(())
    }

    pub async fn get_recently_played(&self, limit: u32) -> Result<Vec<RecentlyPlayedItem>> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let url = format!("https://api.spotify.com/v1/me/player/recently-played?limit={}", limit);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch recently played tracks")?;

        let status = response.status();
        let response_text = response.text().await?;

        println!("DEBUG Recently Played - Status: {}", status);

        let recent_response: RecentlyPlayedResponse = serde_json::from_str(&response_text)
            .context(format!("Failed to parse recently played response: {}", response_text))?;

        Ok(recent_response.items)
    }

    pub async fn get_playlist_tracks(&self, playlist_id: &str) -> Result<Vec<PlaylistTrackItem>> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let url = format!("https://api.spotify.com/v1/playlists/{}/tracks", playlist_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .context("Failed to fetch playlist tracks")?;

        let status = response.status();
        let response_text = response.text().await?;

        println!("DEBUG Playlist Tracks - Status: {}", status);

        let tracks_response: PlaylistTracksResponse = serde_json::from_str(&response_text)
            .context(format!("Failed to parse playlist tracks response: {}", response_text))?;

        Ok(tracks_response.items)
    }

    pub async fn remove_tracks_from_playlist(&self, playlist_id: &str, tracks_with_positions: Vec<(String, usize)>) -> Result<()> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let url = format!("https://api.spotify.com/v1/playlists/{}/tracks", playlist_id);

        // Group positions by URI
        let mut uri_positions: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
        for (uri, position) in tracks_with_positions {
            uri_positions.entry(uri).or_insert_with(Vec::new).push(position);
        }

        // For each unique URI, remove all its positions in one request
        for (uri, positions) in uri_positions {
            let tracks_to_remove = vec![serde_json::json!({
                "uri": uri,
                "positions": positions
            })];

            let body = serde_json::json!({
                "tracks": tracks_to_remove
            });

            println!("DEBUG: Removing track with body: {}", serde_json::to_string_pretty(&body).unwrap());

            let response = self
                .client
                .delete(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .context("Failed to remove tracks from playlist")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await?;
                return Err(anyhow::anyhow!("Failed to remove tracks: {} - {}", status, text));
            }
        }

        Ok(())
    }

    pub async fn get_tracks(&self, track_ids: Vec<String>) -> Result<Vec<Track>> {
        let token = self
            .get_token()
            .await
            .context("No access token available")?;

        let mut all_tracks = Vec::new();

        // Spotify API allows up to 50 tracks per request
        for chunk in track_ids.chunks(50) {
            let ids = chunk.join(",");
            let url = format!("https://api.spotify.com/v1/tracks?ids={}", ids);

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .send()
                .await
                .context("Failed to fetch tracks")?;

            let status = response.status();
            let response_text = response.text().await?;

            if !status.is_success() {
                return Err(anyhow::anyhow!("Failed to fetch tracks: {} - {}", status, response_text));
            }

            #[derive(Deserialize)]
            struct TracksResponse {
                tracks: Vec<Option<Track>>,
            }

            let tracks_response: TracksResponse = serde_json::from_str(&response_text)
                .context(format!("Failed to parse tracks response: {}", response_text))?;

            // Filter out None values (tracks that don't exist or are unavailable)
            all_tracks.extend(tracks_response.tracks.into_iter().filter_map(|t| t));
        }

        Ok(all_tracks)
    }
}