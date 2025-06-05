#!/bin/bash

# Colores para mensajes
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Instalando SpotiGod...${NC}"

# Verificar si cargo está instalado
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo no está instalado. Por favor, instala Rust primero.${NC}"
    exit 1
fi

# Compilar el proyecto
echo "Compilando el proyecto..."
cargo build --release

if [ $? -ne 0 ]; then
    echo -e "${RED}Error al compilar el proyecto${NC}"
    exit 1
fi

# Crear directorio si no existe
sudo mkdir -p /usr/local/bin

# Copiar el binario
echo "Instalando el binario..."
sudo cp target/release/spotigod /usr/local/bin/

# Dar permisos de ejecución
sudo chmod +x /usr/local/bin/spotigod

echo -e "${GREEN}¡Instalación completada!${NC}"
echo "Ahora puedes ejecutar SpotiGod desde cualquier ubicación usando el comando: spotigod" 