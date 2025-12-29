use crate::models::*;

/// Get mock user data for demo mode
pub fn get_mock_user() -> User {
    User {
        display_name: Some("Demo User".to_string()),
        id: "demo_user_123".to_string(),
        email: Some("demo@example.com".to_string()),
        images: vec![],
        followers: Followers { total: 42 },
        country: Some("US".to_string()),
    }
}

/// Get mock top tracks for demo mode
pub fn get_mock_top_tracks() -> Vec<Track> {
    vec![
        Track {
            id: "track1".to_string(),
            name: "Bohemian Rhapsody".to_string(),
            artists: vec![Artist {
                id: "artist1".to_string(),
                name: "Queen".to_string(),
                genres: None,
                images: None,
                external_urls: ExternalUrls {
                    spotify: "https://open.spotify.com".to_string(),
                },
                followers: None,
popularity: None,
            }],
            album: Album {
                id: "album1".to_string(),
                name: "A Night at the Opera".to_string(),
                images: vec![],
                release_date: "1975-11-21".to_string(),
            },
            duration_ms: 354000,
            external_urls: ExternalUrls {
                spotify: "https://open.spotify.com".to_string(),
            },
            popularity: Some(92),
            explicit: false,
        },
        Track {
            id: "track2".to_string(),
            name: "Stairway to Heaven".to_string(),
            artists: vec![Artist {
                id: "artist2".to_string(),
                name: "Led Zeppelin".to_string(),
                genres: None,
                images: None,
                external_urls: ExternalUrls {
                    spotify: "https://open.spotify.com".to_string(),
                },
                followers: None,
popularity: None,
            }],
            album: Album {
                id: "album2".to_string(),
                name: "Led Zeppelin IV".to_string(),
                images: vec![],
                release_date: "1971-11-08".to_string(),
            },
            duration_ms: 482000,
            external_urls: ExternalUrls {
                spotify: "https://open.spotify.com".to_string(),
            },
            popularity: Some(88),
            explicit: false,
        },
        Track {
            id: "track3".to_string(),
            name: "Hotel California".to_string(),
            artists: vec![Artist {
                id: "artist3".to_string(),
                name: "Eagles".to_string(),
                genres: None,
                images: None,
                external_urls: ExternalUrls {
                    spotify: "https://open.spotify.com".to_string(),
                },
                followers: None,
popularity: None,
            }],
            album: Album {
                id: "album3".to_string(),
                name: "Hotel California".to_string(),
                images: vec![],
                release_date: "1976-12-08".to_string(),
            },
            duration_ms: 391000,
            external_urls: ExternalUrls {
                spotify: "https://open.spotify.com".to_string(),
            },
            popularity: Some(90),
            explicit: false,
        },
    ]
}

/// Get mock top artists for demo mode
pub fn get_mock_top_artists() -> Vec<Artist> {
    vec![
        Artist {
            id: "artist1".to_string(),
            name: "Queen".to_string(),
            genres: Some(vec!["classic rock".to_string(), "glam rock".to_string()]),
            images: Some(vec![]),
            external_urls: ExternalUrls {
                spotify: "https://open.spotify.com".to_string(),
            },
            followers: Some(Followers { total: 35000000 }),
popularity: Some(85),
        },
        Artist {
            id: "artist2".to_string(),
            name: "Led Zeppelin".to_string(),
            genres: Some(vec!["hard rock".to_string(), "classic rock".to_string()]),
            images: Some(vec![]),
            external_urls: ExternalUrls {
                spotify: "https://open.spotify.com".to_string(),
            },
            followers: Some(Followers { total: 28000000 }),
popularity: Some(85),
        },
        Artist {
            id: "artist3".to_string(),
            name: "Eagles".to_string(),
            genres: Some(vec!["classic rock".to_string(), "soft rock".to_string()]),
            images: Some(vec![]),
            external_urls: ExternalUrls {
                spotify: "https://open.spotify.com".to_string(),
            },
            followers: Some(Followers { total: 15000000 }),
popularity: Some(85),
        },
    ]
}

/// Get mock playlists for demo mode
pub fn get_mock_playlists() -> Vec<Playlist> {
    vec![
        Playlist {
            id: "playlist1".to_string(),
            name: "Classic Rock Favorites".to_string(),
            description: Some("The best of classic rock".to_string()),
            images: vec![],
            tracks: PlaylistTracks { total: 25 },
            external_urls: ExternalUrls {
                spotify: "https://open.spotify.com".to_string(),
            },
            owner: PlaylistOwner {
                display_name: Some("Demo User".to_string()),
                id: "demo_user_123".to_string(),
            },
            public: Some(true),
        },
        Playlist {
            id: "playlist2".to_string(),
            name: "Road Trip Mix".to_string(),
            description: Some("Perfect songs for long drives".to_string()),
            images: vec![],
            tracks: PlaylistTracks { total: 40 },
            external_urls: ExternalUrls {
                spotify: "https://open.spotify.com".to_string(),
            },
            owner: PlaylistOwner {
                display_name: Some("Demo User".to_string()),
                id: "demo_user_123".to_string(),
            },
            public: Some(false),
        },
    ]
}

/// Get mock recently played tracks for demo mode
pub fn get_mock_recently_played() -> Vec<RecentlyPlayedItem> {
    let tracks = get_mock_top_tracks();
    vec![
        RecentlyPlayedItem {
            track: tracks[0].clone(),
            played_at: "2024-12-26T10:30:00Z".to_string(),
        },
        RecentlyPlayedItem {
            track: tracks[1].clone(),
            played_at: "2024-12-26T09:15:00Z".to_string(),
        },
    ]
}