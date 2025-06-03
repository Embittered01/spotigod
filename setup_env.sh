#!/bin/bash

# Script para configurar las variables de entorno de SpotiGod
# 
# INSTRUCCIONES:
# 1. Ve a https://developer.spotify.com/dashboard
# 2. Crea una nueva app
# 3. Copia el Client ID y Client Secret
# 4. Agrega http://127.0.0.1:8888/callback como Redirect URI
# 5. Ejecuta este script con tus credenciales:
#    ./setup_env.sh tu_client_id tu_client_secret

if [ $# -ne 2 ]; then
    echo "❌ Uso: $0 <client_id> <client_secret>"
    echo ""
    echo "📝 Instrucciones:"
    echo "   1. Ve a https://developer.spotify.com/dashboard"
    echo "   2. Crea una nueva app"
    echo "   3. Copia el Client ID y Client Secret"
    echo "   4. Agrega http://127.0.0.1:8888/callback como Redirect URI"
    echo "   5. Ejecuta: $0 tu_client_id tu_client_secret"
    echo ""
    echo "Ejemplo:"
    echo "   $0 abc123def456 xyz789uvw012"
    exit 1
fi

CLIENT_ID="$1"
CLIENT_SECRET="$2"

# Detectar el shell y archivo de configuración
if [ -n "$ZSH_VERSION" ]; then
    SHELL_CONFIG="$HOME/.zshrc"
    SHELL_NAME="zsh"
elif [ -n "$BASH_VERSION" ]; then
    SHELL_CONFIG="$HOME/.bashrc"
    SHELL_NAME="bash"
else
    SHELL_CONFIG="$HOME/.profile"
    SHELL_NAME="shell"
fi

echo "🔧 Configurando variables de entorno para SpotiGod..."

# Crear backup del archivo de configuración
cp "$SHELL_CONFIG" "$SHELL_CONFIG.backup.$(date +%Y%m%d_%H%M%S)" 2>/dev/null

# Verificar si las variables ya existen
if grep -q "SPOTIFY_CLIENT_ID" "$SHELL_CONFIG" 2>/dev/null; then
    echo "⚠️  Las variables de Spotify ya existen en $SHELL_CONFIG"
    echo "   Si quieres actualizarlas, edita manualmente el archivo o:"
    echo "   1. Elimina las líneas existentes"
    echo "   2. Ejecuta este script de nuevo"
    exit 1
fi

# Agregar las variables al archivo de configuración
echo "" >> "$SHELL_CONFIG"
echo "# SpotiGod - Cliente de Spotify" >> "$SHELL_CONFIG"
echo "export SPOTIFY_CLIENT_ID=\"$CLIENT_ID\"" >> "$SHELL_CONFIG"
echo "export SPOTIFY_CLIENT_SECRET=\"$CLIENT_SECRET\"" >> "$SHELL_CONFIG"

echo "✅ Variables de entorno configuradas en $SHELL_CONFIG"
echo ""
echo "🔄 Para aplicar los cambios, ejecuta:"
echo "   source $SHELL_CONFIG"
echo ""
echo "🚀 Luego puedes ejecutar SpotiGod con:"
echo "   cargo run"

# Ofrecer recargar automáticamente
read -p "¿Quieres recargar las variables de entorno ahora? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    source "$SHELL_CONFIG"
    echo "✅ Variables de entorno recargadas!"
    echo ""
    echo "🎵 ¡Todo listo! Ejecuta 'cargo run' para usar SpotiGod"
fi 