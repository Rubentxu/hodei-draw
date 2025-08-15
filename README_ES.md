# 🎨 Hodei Draw

[![Estado de Build](https://img.shields.io/github/actions/workflow/status/Rubentxu/hodei-draw/deploy.yml?branch=main&label=deployment)](https://github.com/Rubentxu/hodei-draw/actions)
[![Demo en Vivo](https://img.shields.io/badge/Demo%20en%20Vivo-🚀%20Disponible-brightgreen)](https://rubentxu.github.io/hodei-draw/)
[![Licencia: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?logo=webassembly&logoColor=white)](https://webassembly.org/)

> **Aplicación web interactiva de dibujo y diagramación construida con Rust/WASM**  
> Un lienzo estilo Excalidraw con capacidades de animación y simulación física

[📖 **Leer en Inglés**](README.md) | [🚀 **Demo en Vivo**](https://rubentxu.github.io/hodei-draw) | [📋 **Hoja de Ruta**](docs/ROADMAP.md)

---

## ✨ Descripción General

**Hodei Draw** es una pizarra interactiva de nueva generación construida con tecnologías web de vanguardia. Combina la simplicidad y elegancia de Excalidraw con potentes capacidades de animación y simulación física 2D integrada, todo ejecutándose con rendimiento nativo a través de Rust y WebAssembly.

### 🎯 Características Principales

- **🎨 Interfaz de Dibujo Intuitiva** - UI inspirada en Excalidraw con barra de herramientas mejorada y diseño responsivo
- **⚡ Renderizado de Alto Rendimiento** - WebGPU primero con respaldo Canvas2D para máxima compatibilidad
- **🎭 Sistema de Temas** - Soporte completo para temas Claro/Oscuro/Sistema con propiedades CSS personalizadas
- **🔧 Herramientas Multi-Forma** - Herramientas de rectángulo, elipse, línea y selección con vista previa en vivo
- **📱 Diseño Responsivo** - Funciona sin problemas en dispositivos de escritorio y móviles
- **🌐 Stack Web Moderno** - Construido con Rust, WASM, Leptos y Tailwind CSS v4

### 🚀 Características Planificadas (Hoja de Ruta)

- **🎬 Sistema de Animación** - Animación basada en componentes con línea de tiempo y funciones de easing
- **⚛️ Simulación Física** - Motor de física 2D integrado para diagramas interactivos
- **👥 Colaboración en Tiempo Real** - Colaboración P2P con rollback netcode
- **🧩 Sistema de Plugins** - Arquitectura extensible con WASM Component Model
- **📚 Bibliotecas de Componentes** - Bibliotecas pre-construidas para Cloud, UML y diagramas técnicos

---

## 🏗️ Stack Tecnológico

| Componente | Tecnología | Propósito |
|------------|------------|-----------|
| **Framework Frontend** | [Leptos](https://leptos.dev) | UI reactiva con reactividad granular |
| **Lenguaje** | [Rust](https://rust-lang.org) + [WebAssembly](https://webassembly.org) | Ejecución de alto rendimiento y segura en memoria |
| **Renderizado** | [wgpu](https://wgpu.rs) + Canvas2D | Renderizado acelerado por GPU con respaldo |
| **Gestión de Estado** | [Bevy ECS](https://bevyengine.org/learn/book/getting-started/ecs/) | Arquitectura Entity Component System |
| **Estilos** | [Tailwind CSS v4](https://tailwindcss.com) | CSS utility-first con sistema de diseño personalizado |
| **Herramienta de Build** | [Trunk](https://trunkrs.dev) | Herramienta de build enfocada en WASM y servidor dev |

---

## 🚀 Inicio Rápido

### Prerequisitos

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+) 
- [Trunk](https://trunkrs.dev/) - Herramienta de build WASM

```bash
# Instalar Rust (si no está instalado)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Instalar target wasm32
rustup target add wasm32-unknown-unknown

# Instalar Trunk
cargo install trunk

# Instalar dependencias de Node.js (para procesamiento CSS)
cd apps/app-web && npm install
```

### Desarrollo

```bash
# Clonar el repositorio
git clone https://github.com/Rubentxu/hodei-draw.git
cd hodei-draw

# Iniciar servidor de desarrollo
make serve
# O
cd apps/app-web && trunk serve --open --features webgpu
```

La aplicación estará disponible en `http://127.0.0.1:8080`

### Construir para Producción

```bash
# Construir bundle WASM optimizado
make build
# O  
cd apps/app-web && trunk build --release
```

---

## 📁 Estructura del Proyecto

```
hodei-draw/
├── 📁 apps/
│   └── 📁 app-web/          # Aplicación web principal
│       ├── 📄 src/          # Código fuente Rust
│       ├── 📄 index.html    # Plantilla HTML
│       ├── 📄 input.css     # Entrada Tailwind CSS
│       └── 📄 Trunk.toml    # Configuración de build
├── 📁 crates/              # Crates del workspace Rust
│   ├── 📁 core/            # Lógica de negocio central
│   ├── 📁 ecs/             # Entity Component System
│   ├── 📁 ui-leptos/       # Componentes UI Leptos
│   └── 📁 design-system/   # Sistema de diseño y componentes
├── 📁 docs/                # Documentación
├── 📄 Cargo.toml          # Configuración del workspace
├── 📄 Makefile            # Comandos de desarrollo
└── 📄 README.md           # Este archivo
```

### Arquitectura

**Hodei Draw** sigue un patrón de **arquitectura hexagonal** con separación clara de responsabilidades:

- **🎯 Dominio Central** (`crates/core/`) - Lógica de negocio, independiente de UI o renderizado
- **⚙️ Capa ECS** (`crates/ecs/`) - Entity Component System para gestión de estado  
- **🎨 Capa UI** (`crates/ui-leptos/`) - Componentes de interfaz de usuario reactivos
- **🎭 Sistema de Diseño** (`crates/design-system/`) - Componentes UI reutilizables y temas
- **🖼️ Renderizado** (`apps/app-web/src/renderer_*`) - Renderizadores WebGPU y Canvas2D

---

## 🛠️ Comandos de Desarrollo

| Comando | Descripción |
|---------|-------------|
| `make serve` | Iniciar servidor de desarrollo con recarga automática |
| `make build` | Construir bundle de producción optimizado |
| `make test` | Ejecutar todas las pruebas |
| `make lint` | Ejecutar linter clippy |
| `make format` | Formatear código con rustfmt |
| `make clean` | Limpiar artefactos de build |

---

## 🎨 Características en Detalle

### Soporte Multi-Renderizador
- **WebGPU**: Renderizado GPU de alto rendimiento para navegadores modernos
- **Respaldo Canvas2D**: Respaldo automático para mayor compatibilidad de navegadores
- **Cambio en Runtime**: Selección dinámica de renderizador con indicadores visuales

### Sistema de Diseño Reactivo
- **Soporte de Temas**: Temas Claro, Oscuro y Sistema con transiciones suaves
- **Biblioteca de Componentes**: Barra de herramientas, barra lateral y paneles flotantes estilo Excalidraw
- **Layout Responsivo**: Optimizado para interfaces de escritorio y móviles

### Herramientas de Formas y Vista Previa
- **Vista Previa en Vivo**: Vista previa de formas en tiempo real durante operaciones de dibujo
- **Soporte Multi-Forma**: Herramientas de rectángulo, elipse, línea con comportamiento consistente
- **Sistema de Selección**: Herramienta de selección avanzada con gestión de estado reactivo

---

## 📚 Documentación

- [🏗️ **Resumen de Arquitectura**](docs/ARCHITECTURE.md) - Arquitectura técnica y decisiones de diseño
- [🗺️ **Hoja de Ruta**](docs/ROADMAP.md) - Hoja de ruta del proyecto y características planificadas  
- [📋 **Requisitos**](docs/REQUISITOS.md) - Requisitos funcionales y no funcionales detallados
- [🚀 **Plan de Implementación**](docs/IMPLEMENTATION_PLAN_FASES.md) - Enfoque de desarrollo por fases
- [🔌 **Contratos de API**](docs/PORTS_CONTRACTS.md) - Contratos de puertos y adaptadores

---

## 🤝 Contribuir

¡Damos la bienvenida a las contribuciones! Por favor, consulta nuestras [Guías de Contribución](CONTRIBUTING_ES.md) para más detalles.

### Flujo de Trabajo de Desarrollo

1. Haz fork del repositorio
2. Crea una rama de característica (`git checkout -b feature/caracteristica-increible`)
3. Confirma tus cambios (`git commit -m 'feat: agregar característica increíble'`)
4. Push a la rama (`git push origin feature/caracteristica-increible`)
5. Abre un Pull Request

### Estándares de Código

- Sigue [Conventional Commits](https://conventionalcommits.org/) para mensajes de commit
- Usa `cargo fmt` para formateo de código
- Asegúrate de que `cargo clippy` pase sin advertencias
- Agrega pruebas para nueva funcionalidad

---

## 🗺️ Hoja de Ruta

### Fase 1: Base de Dibujo Mejorada ✅
- [x] UI estilo Excalidraw con sistema de diseño moderno
- [x] Arquitectura multi-renderizador (WebGPU + Canvas2D)
- [x] Herramientas de formas básicas con vista previa en vivo
- [x] Sistema de temas y diseño responsivo

### Fase 2: Lienzo Interactivo 🚧
- [ ] Sistema de animación basado en componentes
- [ ] Integración de motor de física 2D (Avian/Rapier)
- [ ] Gestión de línea de tiempo y keyframes
- [ ] Bibliotecas de formas avanzadas

### Fase 3: Plataforma de Colaboración 🔮
- [ ] Colaboración en tiempo real (WebRTC + GGRS)
- [ ] Sistema de plugins (WASM Component Model)
- [ ] Sincronización en la nube
- [ ] Características de workspace de equipo

---

## 📄 Licencia

Este proyecto está licenciado bajo la Licencia MIT - consulta el archivo [LICENSE](LICENSE) para más detalles.

---

## 🙏 Agradecimientos

- [Excalidraw](https://excalidraw.com/) - Inspiración para el diseño de interfaz de usuario
- [Bevy Engine](https://bevyengine.org/) - Patrones de arquitectura ECS
- [Leptos](https://leptos.dev/) - Framework web reactivo
- [wgpu](https://wgpu.rs/) - API de gráficos moderna

---

<div align="center">

**[⭐ Dale estrella a este repo](https://github.com/Rubentxu/hodei-draw/stargazers)** • **[🐛 Reportar Bug](https://github.com/Rubentxu/hodei-draw/issues)** • **[💡 Solicitar Característica](https://github.com/Rubentxu/hodei-draw/issues)**

Hecho con ❤️ por [Rubentxu](https://github.com/Rubentxu)

</div>