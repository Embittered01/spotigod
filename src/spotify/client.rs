use super::models::{TokenResponse, PlaybackState, SearchResults, UserProfile, PlaylistsResponse, Track, SavedTracksResponse, PlaylistTracksResponse};
use crate::config::Config;
use anyhow::{anyhow, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as Base64};
use reqwest::Client;
use serde_json::json;
use std::io::prelude::*;
use url::Url;
use uuid::Uuid;

pub struct SpotifyClient {
    client: Client,
    config: Config,
    base_url: String,
}

impl SpotifyClient {
    pub fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
            base_url: "https://api.spotify.com/v1".to_string(),
        }
    }

    pub async fn is_authenticated(&self) -> bool {
        self.config.is_token_valid()
    }

    pub async fn authenticate(&mut self) -> Result<()> {
        // Generar state para OAuth
        let state = Uuid::new_v4().to_string();
        
        // Construir URL de autorización
        let auth_url = format!(
            "https://accounts.spotify.com/authorize?response_type=code&client_id={}&scope={}&redirect_uri={}&state={}",
            self.config.client_id,
            "user-read-playback-state user-modify-playback-state user-read-currently-playing playlist-read-private playlist-read-collaborative user-library-read user-library-modify",
            urlencoding::encode(&self.config.redirect_uri),
            state
        );

        println!("{}", "🌐 Abriendo navegador para autenticación...");
        println!("{}", "📋 Si no se abre automáticamente, copia esta URL:");
        println!("{}", &auth_url);
        
        // Intentar abrir el navegador
        if let Err(_) = webbrowser::open(&auth_url) {
            println!("{}", "⚠️  No se pudo abrir el navegador automáticamente");
        }

        // Iniciar servidor temporal para recibir el callback
        let code = self.start_callback_server().await?;
        
        // Intercambiar código por token
        self.exchange_code_for_token(&code).await?;
        
        Ok(())
    }

    async fn start_callback_server(&self) -> Result<String> {
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:8888")?;
        println!("{}", "🔄 Esperando callback de Spotify...");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0; 1024];
                    stream.read(&mut buffer)?;
                    
                    let request = String::from_utf8_lossy(&buffer[..]);
                    if let Some(line) = request.lines().next() {
                        if line.starts_with("GET") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() > 1 {
                                let url_part = parts[1];
                                if url_part.contains("code=") {
                                    // Extraer el código
                                    let url = format!("http://127.0.0.1:8888{}", url_part);
                                    let parsed_url = Url::parse(&url)?;
                                    let code = parsed_url
                                        .query_pairs()
                                        .find(|(key, _)| key == "code")
                                        .map(|(_, value)| value.to_string())
                                        .ok_or_else(|| anyhow!("No se encontró el código en la respuesta"))?;

                                    // Responder al navegador
                                    let response = "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>¡Autenticación exitosa!</h1><p>Puedes cerrar esta ventana y volver a la terminal.</p></body></html>";
                                    stream.write_all(response.as_bytes())?;
                                    stream.flush()?;
                                    
                                    return Ok(code);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error en conexión: {}", e);
                }
            }
        }
        
        Err(anyhow!("No se recibió el callback de autenticación"))
    }

    async fn exchange_code_for_token(&mut self, code: &str) -> Result<()> {
        let auth_header = Base64.encode(format!("{}:{}", self.config.client_id, self.config.client_secret));
        
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
        ];

        let response = self.client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", auth_header))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: TokenResponse = response.json().await?;
            
            self.config.access_token = Some(token_response.access_token);
            self.config.refresh_token = token_response.refresh_token;
            self.config.token_expires_at = Some(
                chrono::Utc::now().timestamp() + token_response.expires_in
            );
            
            self.config.save().await?;
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow!("Error al obtener token: {}", error_text))
        }
    }

    async fn ensure_valid_token(&mut self) -> Result<()> {
        if !self.config.is_token_valid() {
            if let Some(refresh_token) = self.config.refresh_token.clone() {
                self.refresh_access_token(&refresh_token).await?;
            } else {
                return Err(anyhow!("Token expirado y no hay refresh token. Necesitas autenticarte de nuevo."));
            }
        }
        Ok(())
    }

    async fn refresh_access_token(&mut self, refresh_token: &str) -> Result<()> {
        let auth_header = Base64.encode(format!("{}:{}", self.config.client_id, self.config.client_secret));
        
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ];

        let response = self.client
            .post("https://accounts.spotify.com/api/token")
            .header("Authorization", format!("Basic {}", auth_header))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let token_response: TokenResponse = response.json().await?;
            
            self.config.access_token = Some(token_response.access_token);
            if let Some(new_refresh_token) = token_response.refresh_token {
                self.config.refresh_token = Some(new_refresh_token);
            }
            self.config.token_expires_at = Some(
                chrono::Utc::now().timestamp() + token_response.expires_in
            );
            
            self.config.save().await?;
            Ok(())
        } else {
            Err(anyhow!("Error al refrescar token"))
        }
    }

    async fn get_auth_header(&mut self) -> Result<String> {
        self.ensure_valid_token().await?;
        let token = self.config.access_token.as_ref()
            .ok_or_else(|| anyhow!("No hay token de acceso"))?;
        Ok(format!("Bearer {}", token))
    }

    // Métodos para interactuar con la API de Spotify

    pub async fn get_current_playback(&mut self) -> Result<Option<PlaybackState>> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .get(&format!("{}/me/player", self.base_url))
            .header("Authorization", auth_header)
            .send()
            .await?;

        if response.status() == 204 {
            // No hay reproducción activa
            return Ok(None);
        }

        if response.status().is_success() {
            let playback_state: PlaybackState = response.json().await?;
            Ok(Some(playback_state))
        } else {
            Err(anyhow!("Error al obtener estado de reproducción: {}", response.status()))
        }
    }

    pub async fn play(&mut self) -> Result<()> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .put(&format!("{}/me/player/play", self.base_url))
            .header("Authorization", auth_header)
            .header("Content-Length", "0")
            .body("")
            .send()
            .await?;

        if response.status().is_success() || response.status() == 204 {
            Ok(())
        } else {
            Err(anyhow!("Error al reproducir: {}", response.status()))
        }
    }

    pub async fn pause(&mut self) -> Result<()> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .put(&format!("{}/me/player/pause", self.base_url))
            .header("Authorization", auth_header)
            .header("Content-Length", "0")
            .body("")
            .send()
            .await?;

        if response.status().is_success() || response.status() == 204 {
            Ok(())
        } else {
            Err(anyhow!("Error al pausar: {}", response.status()))
        }
    }

    pub async fn next_track(&mut self) -> Result<()> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .post(&format!("{}/me/player/next", self.base_url))
            .header("Authorization", auth_header)
            .header("Content-Length", "0")
            .body("")
            .send()
            .await?;

        if response.status().is_success() || response.status() == 204 {
            Ok(())
        } else {
            Err(anyhow!("Error al saltar a siguiente canción: {}", response.status()))
        }
    }

    pub async fn previous_track(&mut self) -> Result<()> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .post(&format!("{}/me/player/previous", self.base_url))
            .header("Authorization", auth_header)
            .header("Content-Length", "0")
            .body("")
            .send()
            .await?;

        if response.status().is_success() || response.status() == 204 {
            Ok(())
        } else {
            Err(anyhow!("Error al ir a canción anterior: {}", response.status()))
        }
    }

    pub async fn set_volume(&mut self, volume: u8) -> Result<()> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .put(&format!("{}/me/player/volume?volume_percent={}", self.base_url, volume))
            .header("Authorization", auth_header)
            .header("Content-Length", "0")
            .body("")
            .send()
            .await?;

        if response.status().is_success() || response.status() == 204 {
            Ok(())
        } else {
            Err(anyhow!("Error al cambiar volumen: {}", response.status()))
        }
    }

    pub async fn search_tracks(&mut self, query: &str, limit: u8) -> Result<Vec<Track>> {
        let auth_header = self.get_auth_header().await?;
        let encoded_query = urlencoding::encode(query);
        
        let response = self.client
            .get(&format!("{}/search?q={}&type=track&limit={}", self.base_url, encoded_query, limit))
            .header("Authorization", auth_header)
            .send()
            .await?;

        if response.status().is_success() {
            let search_results: SearchResults = response.json().await?;
            Ok(search_results.tracks.map(|t| t.items).unwrap_or_default())
        } else {
            Err(anyhow!("Error en búsqueda: {}", response.status()))
        }
    }

    pub async fn play_track(&mut self, track_uri: &str) -> Result<()> {
        let auth_header = self.get_auth_header().await?;
        
        let body = json!({
            "uris": [track_uri]
        });

        let response = self.client
            .put(&format!("{}/me/player/play", self.base_url))
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() || response.status() == 204 {
            Ok(())
        } else {
            Err(anyhow!("Error al reproducir canción: {}", response.status()))
        }
    }

    pub async fn get_user_profile(&mut self) -> Result<UserProfile> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .get(&format!("{}/me", self.base_url))
            .header("Authorization", auth_header)
            .send()
            .await?;

        if response.status().is_success() {
            let profile: UserProfile = response.json().await?;
            Ok(profile)
        } else {
            Err(anyhow!("Error al obtener perfil de usuario: {}", response.status()))
        }
    }

    pub async fn get_user_playlists(&mut self) -> Result<Vec<crate::spotify::models::Playlist>> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .get(&format!("{}/me/playlists?limit=50", self.base_url))
            .header("Authorization", auth_header)
            .send()
            .await?;

        if response.status().is_success() {
            let playlists_response: PlaylistsResponse = response.json().await?;
            Ok(playlists_response.items)
        } else {
            Err(anyhow!("Error al obtener playlists: {}", response.status()))
        }
    }

    pub async fn get_saved_tracks(&mut self) -> Result<Vec<Track>> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .get(&format!("{}/me/tracks?limit=50", self.base_url))
            .header("Authorization", auth_header)
            .send()
            .await?;

        if response.status().is_success() {
            let saved_tracks: SavedTracksResponse = response.json().await?;
            Ok(saved_tracks.items.into_iter().map(|item| item.track).collect())
        } else {
            Err(anyhow!("Error al obtener canciones favoritas: {}", response.status()))
        }
    }

    pub async fn get_playlist_tracks(&mut self, playlist_id: &str) -> Result<Vec<Track>> {
        let auth_header = self.get_auth_header().await?;
        
        let response = self.client
            .get(&format!("{}/playlists/{}/tracks?limit=50", self.base_url, playlist_id))
            .header("Authorization", auth_header)
            .send()
            .await?;

        if response.status().is_success() {
            let playlist_tracks: PlaylistTracksResponse = response.json().await?;
            Ok(playlist_tracks.items.into_iter().filter_map(|item| item.track).collect())
        } else {
            Err(anyhow!("Error al obtener canciones de la playlist: {}", response.status()))
        }
    }

    pub async fn play_playlist(&mut self, playlist_uri: &str) -> Result<()> {
        let auth_header = self.get_auth_header().await?;
        
        let body = json!({
            "context_uri": playlist_uri
        });

        let response = self.client
            .put(&format!("{}/me/player/play", self.base_url))
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() || response.status() == 204 {
            Ok(())
        } else {
            Err(anyhow!("Error al reproducir playlist: {}", response.status()))
        }
    }

    pub async fn toggle_shuffle(&mut self) -> Result<()> {
        // Primero obtenemos el estado actual
        if let Some(current_state) = self.get_current_playback().await? {
            let new_shuffle_state = !current_state.shuffle_state;
            let auth_header = self.get_auth_header().await?;
            
            let response = self.client
                .put(&format!("{}/me/player/shuffle?state={}", self.base_url, new_shuffle_state))
                .header("Authorization", auth_header)
                .send()
                .await?;

            if response.status().is_success() || response.status() == 204 {
                Ok(())
            } else {
                Err(anyhow!("Error al cambiar shuffle: {}", response.status()))
            }
        } else {
            Err(anyhow!("No hay reproducción activa"))
        }
    }

    pub async fn toggle_repeat(&mut self) -> Result<()> {
        // Ciclar entre off -> context -> track -> off
        if let Some(current_state) = self.get_current_playback().await? {
            let new_repeat_state = match current_state.repeat_state.as_str() {
                "off" => "context",
                "context" => "track", 
                "track" => "off",
                _ => "off",
            };
            
            let auth_header = self.get_auth_header().await?;
            
            let response = self.client
                .put(&format!("{}/me/player/repeat?state={}", self.base_url, new_repeat_state))
                .header("Authorization", auth_header)
                .send()
                .await?;

            if response.status().is_success() || response.status() == 204 {
                Ok(())
            } else {
                Err(anyhow!("Error al cambiar repeat: {}", response.status()))
            }
        } else {
            Err(anyhow!("No hay reproducción activa"))
        }
    }
} 