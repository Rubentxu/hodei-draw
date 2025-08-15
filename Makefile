.PHONY: help serve build test lint clean

help:
	@echo "Available targets:"
	@echo "  make serve  - Serve the web app in development (trunk) and open browser"
	@echo "  make build  - Build the web app for production (trunk build --release)"
	@echo "  make test   - Run workspace tests"
	@echo "  make lint   - Run clippy across the workspace with warnings as errors"
	@echo "  make clean  - Clean cargo artifacts"

serve:
	@echo "🚀 Sirviendo la aplicación en modo desarrollo..."
	@echo "📝 Compilando CSS con Tailwind..."
	@cd apps/app-web && npm run build-css-once
	@echo "🌐 Iniciando servidor de desarrollo..."
	@cd apps/app-web && trunk serve --open --features webgpu

build:
	@echo "📦 Compilando la aplicación para producción..."
	@cd apps/app-web && trunk build --release

test:
	@echo "🧪 Ejecutando tests del workspace..."
	@cargo test --workspace

lint:
	@echo "✨ Verificando el código con Clippy..."
	@cargo clippy --workspace --all-targets --all-features -- -D warnings

clean:
	@echo "🧹 Limpiando TODO: artefactos de compilación, caché de trunk y directorio dist..."
	# Limpia los artefactos de compilación de Rust (directorio /target)
	@cargo clean
	# Limpia la caché de Trunk y elimina explícitamente el directorio de salida por si acaso
	@cd apps/app-web && trunk clean && rm -rf dist
	-@fuser -k 8080/tcp 8081/tcp
