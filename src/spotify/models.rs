use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub artists: Vec<Artist>,
    pub album: Album,
    pub duration_ms: i64,
    pub explicit: bool,
    pub external_urls: ExternalUrls,
    pub popularity: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub external_urls: ExternalUrls,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artists: Vec<Artist>,
    pub images: Vec<Image>,
    pub release_date: String,
    pub external_urls: ExternalUrls,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Image {
    pub height: Option<i32>,
    pub url: String,
    pub width: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaybackState {
    pub device: Device,
    pub repeat_state: String,
    pub shuffle_state: bool,
    pub context: Option<Context>,
    pub timestamp: i64,
    pub progress_ms: Option<i64>,
    pub is_playing: bool,
    pub item: Option<Track>,
    pub currently_playing_type: String,
    pub actions: Actions,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Device {
    pub id: Option<String>,
    pub is_active: bool,
    pub is_private_session: bool,
    pub is_restricted: bool,
    pub name: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub volume_percent: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Context {
    pub external_urls: ExternalUrls,
    pub href: String,
    #[serde(rename = "type")]
    pub context_type: String,
    pub uri: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Actions {
    pub interrupting_playback: Option<bool>,
    pub pausing: Option<bool>,
    pub resuming: Option<bool>,
    pub seeking: Option<bool>,
    pub skipping_next: Option<bool>,
    pub skipping_prev: Option<bool>,
    pub toggling_repeat_context: Option<bool>,
    pub toggling_shuffle: Option<bool>,
    pub toggling_repeat_track: Option<bool>,
    pub transferring_playback: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResults {
    pub tracks: Option<TrackSearchResult>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrackSearchResult {
    pub href: String,
    pub items: Vec<Track>,
    pub limit: i32,
    pub next: Option<String>,
    pub offset: i32,
    pub previous: Option<String>,
    pub total: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserProfile {
    pub id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub country: Option<String>,
    pub followers: Followers,
    pub images: Vec<Image>,
    pub product: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Followers {
    pub href: Option<String>,
    pub total: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub images: Vec<Image>,
    pub owner: PlaylistOwner,
    pub public: Option<bool>,
    pub tracks: PlaylistTracks,
    pub external_urls: ExternalUrls,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaylistOwner {
    pub id: String,
    pub display_name: Option<String>,
    pub external_urls: ExternalUrls,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaylistTracks {
    pub href: String,
    pub total: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaylistsResponse {
    pub href: String,
    pub items: Vec<Playlist>,
    pub limit: i32,
    pub next: Option<String>,
    pub offset: i32,
    pub previous: Option<String>,
    pub total: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SavedTracksResponse {
    pub href: String,
    pub items: Vec<SavedTrackItem>,
    pub limit: i32,
    pub next: Option<String>,
    pub offset: i32,
    pub previous: Option<String>,
    pub total: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SavedTrackItem {
    pub added_at: String,
    pub track: Track,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaylistTracksResponse {
    pub href: String,
    pub items: Vec<PlaylistTrackItem>,
    pub limit: i32,
    pub next: Option<String>,
    pub offset: i32,
    pub previous: Option<String>,
    pub total: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaylistTrackItem {
    pub added_at: String,
    pub track: Option<Track>,
} 