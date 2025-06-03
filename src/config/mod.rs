use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<i64>,
}

impl Config {
    pub async fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // Primera vez, crear configuración con valores por defecto
            let config = Config {
                client_id: std::env::var("SPOTIFY_CLIENT_ID").unwrap_or_else(|_| {
                    println!("⚠️  No se encontró SPOTIFY_CLIENT_ID en las variables de entorno");
                    println!("📝 Por favor, ve a https://developer.spotify.com/dashboard");
                    println!("   1. Crea una nueva app");
                    println!("   2. Copia el Client ID y Client Secret");
                    println!("   3. Agrega http://localhost:8888/callback como Redirect URI");
                    println!("   4. Ejecuta: export SPOTIFY_CLIENT_ID=tu_client_id");
                    println!("   5. Ejecuta: export SPOTIFY_CLIENT_SECRET=tu_client_secret");
                    std::process::exit(1);
                }),
                client_secret: std::env::var("SPOTIFY_CLIENT_SECRET").unwrap_or_else(|_| {
                    println!("⚠️  No se encontró SPOTIFY_CLIENT_SECRET en las variables de entorno");
                    std::process::exit(1);
                }),
                redirect_uri: "http://127.0.0.1:8888/callback".to_string(),
                access_token: None,
                refresh_token: None,
                token_expires_at: None,
            };
            
            config.save().await?;
            Ok(config)
        }
    }
    
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
    
    fn get_config_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("No se pudo determinar el directorio home"))?;
        
        Ok(home_dir.join(".config").join("spotigod").join("config.json"))
    }
    
    pub fn is_token_valid(&self) -> bool {
        if let (Some(_), Some(expires_at)) = (&self.access_token, self.token_expires_at) {
            let now = chrono::Utc::now().timestamp();
            expires_at > now + 60 // Token válido si expira en más de 1 minuto
        } else {
            false
        }
    }
} 