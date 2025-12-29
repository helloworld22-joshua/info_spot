use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub display_name: Option<String>,
    pub id: String,
    pub email: Option<String>,
    pub images: Vec<Image>,
    pub followers: Followers,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Followers {
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub duration_ms: u32,
    pub external_urls: ExternalUrls,
    pub popularity: Option<u32>,
    pub explicit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub genres: Option<Vec<String>>,
    pub images: Option<Vec<Image>>,
    pub external_urls: ExternalUrls,
    pub followers: Option<Followers>,
    pub popularity: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub images: Vec<Image>,
    pub release_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub images: Vec<Image>,
    pub tracks: PlaylistTracks,
    pub external_urls: ExternalUrls,
    pub owner: PlaylistOwner,
    pub public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaylistTracks {
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaylistOwner {
    pub display_name: Option<String>,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopTracksResponse {
    pub items: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopArtistsResponse {
    pub items: Vec<Artist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistsResponse {
    pub items: Vec<Playlist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTracksResponse {
    pub items: Vec<PlaylistTrackItem>,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrackItem {
    pub added_at: String,
    pub track: Track,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentlyPlayedResponse {
    pub items: Vec<RecentlyPlayedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentlyPlayedItem {
    pub track: Track,
    pub played_at: String,
}