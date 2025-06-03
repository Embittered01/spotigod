mod spotify;
mod ui;
mod config;

use anyhow::Result;
use colored::Colorize;

use config::Config;
use spotify::SpotifyClient;
use ui::App;

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}", "🎵 Bienvenido a SpotiGod - Tu cliente de Spotify en terminal 🎵".bright_green().bold());
    
    // Cargar configuración
    let config = Config::load().await?;
    
    // Inicializar cliente de Spotify
    let mut spotify_client = SpotifyClient::new(config.clone());
    
    // Verificar si ya tenemos un token válido
    if !spotify_client.is_authenticated().await {
        println!("{}", "🔐 Necesitas autenticarte con Spotify...".yellow());
        spotify_client.authenticate().await?;
        println!("{}", "✅ Autenticación exitosa!".green());
    }
    
    // Inicializar la aplicación TUI
    let mut app = App::new(spotify_client);
    
    // Ejecutar la aplicación
    app.run().await?;
    
    Ok(())
} 