# SpotiGod 🎵

Un cliente de Spotify en terminal escrito en Rust, con una interfaz de usuario moderna y funcionalidades completas.

## Características ✨

- 🎵 Reproducción de música en tiempo real
- 🔍 Búsqueda de canciones
- 📋 Gestión de playlists
- ⭐ Canciones favoritas
- 🎛️ Control de volumen
- 🔀 Modo shuffle
- 🔁 Modo repetición
- 📱 Compatible con cualquier dispositivo de Spotify

## Requisitos 📋

- Rust (última versión estable)
- Una cuenta de Spotify
- Una aplicación registrada en [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)

## Configuración ⚙️

1. Crea una aplicación en [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
2. Obtén el Client ID y Client Secret
3. Configura la URI de redirección como `http://localhost:8888`
4. Crea un archivo `config.json` con la siguiente estructura:

```json
{
    "client_id": "tu_client_id",
    "client_secret": "tu_client_secret",
    "redirect_uri": "http://localhost:8888"
}
```

## Instalación 🚀

```bash
# Clonar el repositorio
git clone https://github.com/tu-usuario/spotigod.git
cd spotigod

# Compilar el proyecto
cargo build --release

# Ejecutar
cargo run --release
```

## Controles 🎮

- `1`: Reproductor
- `2`: Búsqueda
- `3`: Playlists
- `4`: Favoritos
- `Espacio`: Play/Pause
- `←/p`: Canción anterior
- `→/n`: Siguiente canción
- `s`: Shuffle
- `r`: Repeat
- `v`: Volumen
- `/`: Buscar
- `q`: Salir

## Contribuir 🤝

Las contribuciones son bienvenidas. Por favor, abre un issue para discutir los cambios que te gustaría hacer.

## Licencia 📄

Este proyecto está bajo la Licencia MIT. Ver el archivo `LICENSE` para más detalles. 