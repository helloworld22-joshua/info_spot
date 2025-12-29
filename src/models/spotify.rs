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
#[serde(default)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub images: Vec<Image>,
    pub tracks: PlaylistTracks,
    pub external_urls: ExternalUrls,
    pub owner: PlaylistOwner,
    pub public: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collaborative: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub playlist_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

// Custom deserializer to convert null to empty Vec
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: None,
            images: Vec::new(),
            tracks: PlaylistTracks::default(),
            external_urls: ExternalUrls { spotify: String::new() },
            owner: PlaylistOwner::default(),
            public: None,
            collaborative: None,
            href: None,
            primary_color: None,
            snapshot_id: None,
            playlist_type: None,
            uri: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlaylistTracks {
    #[serde(default)]
    pub total: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

impl Default for PlaylistTracks {
    fn default() -> Self {
        Self {
            total: 0,
            href: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct PlaylistOwner {
    pub display_name: Option<String>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_urls: Option<ExternalUrls>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub owner_type: Option<String>,
}

impl Default for PlaylistOwner {
    fn default() -> Self {
        Self {
            display_name: None,
            id: String::new(),
            external_urls: None,
            href: None,
            uri: None,
            owner_type: None,
        }
    }
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