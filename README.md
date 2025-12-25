# InfoSpot 🎵

A beautiful desktop application built with Dioxus (Rust) that connects to Spotify's API to display your music statistics, top tracks, favorite artists, and playlists.

## Features

- 🎨 **Clean UI** - Spotify-inspired design using SASS
- 👤 **User Profile** - Display your Spotify profile with follower count and profile picture
- 🎵 **Top Tracks** - See your most listened to tracks with album art
- 🎤 **Top Artists** - View your favorite artists with genres and follower counts
- 📚 **Playlists** - Browse all your playlists with track counts and descriptions
- ⏱️ **Time Ranges** - Switch between Last 4 Weeks, Last 6 Months, and All Time statistics

## Prerequisites

- Rust (latest stable version)
- A Spotify account
- Spotify Developer App credentials

## Setup

### 1. Get Spotify API Credentials

1. Go to [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
2. Log in with your Spotify account
3. Click "Create App"
4. Fill in the app details:
   - App Name: InfoSpot (or any name you prefer)
   - App Description: Personal music stats viewer
   - Redirect URI: `http://localhost:8888/callback`
5. Accept the terms and create the app
6. Copy your **Client ID** and **Client Secret**

### 2. Configure Environment Variables

1. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` and add your credentials:
   ```env
   SPOTIFY_CLIENT_ID=your_client_id_here
   SPOTIFY_CLIENT_SECRET=your_client_secret_here
   SPOTIFY_REDIRECT_URI=http://localhost:8888/callback
   ```

### 3. Install Dependencies and Build

```bash
# Build the project
cargo build --release

# Or run directly in development mode
cargo run
```

### 4. (Optional) Compile SASS

If you want to modify the styles, compile SASS to CSS:

```bash
# Install sass if you haven't already
npm install -g sass

# Compile SASS
sass assets/styling/style.scss assets/styling/style.css

# Or watch for changes
sass --watch assets/styling/style.scss:assets/styling/style.css
```

## Usage

1. Run the application:
   ```bash
   cargo run --release
   ```

2. Click "Login with Spotify" - your browser will open
3. Authorize the application
4. You'll be redirected back to the app automatically
5. Explore your music stats!

## Project Structure

```
info_spot/
├── src/
│   ├── api/
│   │   ├── mod.rs
│   │   └── spotify.rs        # Spotify API client
│   ├── components/
│   │   ├── mod.rs
│   │   ├── user_profile.rs   # User profile component
│   │   ├── top_tracks.rs     # Top tracks list
│   │   ├── top_artists.rs    # Top artists grid
│   │   └── playlists.rs      # Playlists grid
│   ├── models/
│   │   ├── mod.rs
│   │   └── spotify.rs        # Data models
│   ├── oauth.rs              # OAuth callback server
│   └── main.rs               # App entry point & routes
├── assets/styling/
│   ├── style.scss            # SASS source
│   └── style.css             # Compiled CSS
├── Cargo.toml
├── .env.example
└── README.md
```

## Architecture

### Clean Modular Design

The app follows a clean architecture with separation of concerns:

- **API Layer** (`api/`): Handles all Spotify API communication
- **Components** (`components/`): Reusable UI components for different data views
- **Models** (`models/`): Type-safe data structures using Serde
- **OAuth** (`oauth.rs`): Standalone OAuth flow handling
- **Styling** (`assets/`): SASS-based styling with variables and mixins

### Key Features

- **Type Safety**: Full Rust type safety with Serde for API responses
- **Async Operations**: Uses Tokio for non-blocking API calls
- **State Management**: Dioxus signals for reactive UI updates
- **Routing**: Dioxus router for navigation between views
- **Desktop Native**: True desktop app using Dioxus desktop renderer

## Customization

### Styling

Edit `assets/style.scss` to customize:

- Colors (Spotify green theme by default)
- Layout and spacing
- Component styles
- Responsive breakpoints

Then recompile:
```bash
sass assets/styling/style.scss assets/styling/style.css
```

### API Limits

Adjust the number of items fetched in `src/main.rs`:

```rust
// Dashboard component
client.get_top_tracks(20, &time_range)  // Change 20 to your preferred limit
client.get_top_artists(20, &time_range)
client.get_playlists(50)
```

## Troubleshooting

### "No access token available" error

Make sure you've completed the OAuth flow by clicking "Login with Spotify" and authorizing the app.

### Port 8888 already in use

Change the port in both:
- `.env` file (`SPOTIFY_REDIRECT_URI`)
- Spotify Developer Dashboard (Redirect URIs)
- `src/oauth.rs` (line with `TcpListener::bind`)

### SASS compilation issues

If you don't have SASS installed, the pre-compiled CSS is already included. You only need SASS if you want to modify styles.

## API Rate Limits

Spotify API has rate limits. The app makes the following calls:
- 1 call for user profile
- 1 call per time range change for top tracks
- 1 call per time range change for top artists
- 1 call for playlists

Normal usage should stay well within limits.

## License

MIT License - Feel free to use and modify as you wish!

## Credits

Built with:
- [Dioxus](https://dioxuslabs.com/) - Rust UI framework
- [Spotify Web API](https://developer.spotify.com/documentation/web-api/) - Music data
- SASS - Styling

---

Enjoy exploring your music stats! 🎵
