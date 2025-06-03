.PHONY: help build run release clean test setup check install

# Colores para output
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m # No Color

# Targets por defecto
help: ## Mostrar esta ayuda
	@echo "$(GREEN)🎵 SpotiGod - Cliente de Spotify para Terminal$(NC)"
	@echo ""
	@echo "$(YELLOW)Comandos disponibles:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-15s$(NC) %s\n", $$1, $$2}'
	@echo ""
	@echo "$(YELLOW)Configuración inicial:$(NC)"
	@echo "  1. Crea una app en https://developer.spotify.com/dashboard"
	@echo "  2. Ejecuta: ./setup_env.sh <client_id> <client_secret>"
	@echo "  3. Ejecuta: make run"

setup: ## Mostrar instrucciones de configuración
	@echo "$(GREEN)🔧 Configuración de SpotiGod$(NC)"
	@echo ""
	@echo "$(YELLOW)Pasos para configurar:$(NC)"
	@echo "  1. Ve a https://developer.spotify.com/dashboard"
	@echo "  2. Haz clic en 'Create an App'"
	@echo "  3. Completa el formulario y crea la app"
	@echo "  4. Ve a Settings y copia el Client ID y Client Secret"
	@echo "  5. Agrega 'http://localhost:8888/callback' como Redirect URI"
	@echo "  6. Ejecuta: ./setup_env.sh <tu_client_id> <tu_client_secret>"
	@echo ""
	@echo "$(GREEN)Ejemplo:$(NC)"
	@echo "  ./setup_env.sh abc123def456 xyz789uvw012"

install: ## Instalar dependencias de Rust (si es necesario)
	@echo "$(GREEN)🦀 Verificando instalación de Rust...$(NC)"
	@if ! command -v cargo &> /dev/null; then \
		echo "$(RED)❌ Rust no está instalado$(NC)"; \
		echo "$(YELLOW)Instalando Rust...$(NC)"; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		source $$HOME/.cargo/env; \
	else \
		echo "$(GREEN)✅ Rust ya está instalado$(NC)"; \
	fi

check: ## Verificar que el código compile
	@echo "$(GREEN)🔍 Verificando código...$(NC)"
	@source $$HOME/.cargo/env && cargo check

build: ## Compilar la aplicación en modo debug
	@echo "$(GREEN)🔨 Compilando SpotiGod...$(NC)"
	@source $$HOME/.cargo/env && cargo build

release: ## Compilar la aplicación en modo release (optimizada)
	@echo "$(GREEN)🚀 Compilando SpotiGod (release)...$(NC)"
	@source $$HOME/.cargo/env && cargo build --release
	@echo "$(GREEN)✅ Binario optimizado creado en ./target/release/SpotiGod$(NC)"

run: ## Ejecutar la aplicación
	@echo "$(GREEN)🎵 Ejecutando SpotiGod...$(NC)"
	@if [ -z "$$SPOTIFY_CLIENT_ID" ] || [ -z "$$SPOTIFY_CLIENT_SECRET" ]; then \
		echo "$(RED)❌ Variables de entorno no configuradas$(NC)"; \
		echo "$(YELLOW)Ejecuta: make setup$(NC)"; \
		exit 1; \
	fi
	@source $$HOME/.cargo/env && cargo run

test: ## Ejecutar tests (si los hay)
	@echo "$(GREEN)🧪 Ejecutando tests...$(NC)"
	@source $$HOME/.cargo/env && cargo test

clean: ## Limpiar archivos de compilación
	@echo "$(GREEN)🧹 Limpiando archivos de compilación...$(NC)"
	@source $$HOME/.cargo/env && cargo clean

lint: ## Verificar estilo de código
	@echo "$(GREEN)📝 Verificando estilo de código...$(NC)"
	@source $$HOME/.cargo/env && cargo clippy -- -D warnings

format: ## Formatear código
	@echo "$(GREEN)✨ Formateando código...$(NC)"
	@source $$HOME/.cargo/env && cargo fmt

dev: build ## Compilar y ejecutar en modo desarrollo
	@make run

all: clean build ## Limpiar y compilar todo

env-check: ## Verificar variables de entorno
	@echo "$(GREEN)🔍 Verificando variables de entorno...$(NC)"
	@if [ -n "$$SPOTIFY_CLIENT_ID" ]; then \
		echo "$(GREEN)✅ SPOTIFY_CLIENT_ID configurado$(NC)"; \
	else \
		echo "$(RED)❌ SPOTIFY_CLIENT_ID no configurado$(NC)"; \
	fi
	@if [ -n "$$SPOTIFY_CLIENT_SECRET" ]; then \
		echo "$(GREEN)✅ SPOTIFY_CLIENT_SECRET configurado$(NC)"; \
	else \
		echo "$(RED)❌ SPOTIFY_CLIENT_SECRET no configurado$(NC)"; \
	fi

# Mostrar ayuda por defecto
.DEFAULT_GOAL := help 