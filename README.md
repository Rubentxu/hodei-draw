# 🎨 Hodei Draw

[![Build Status](https://img.shields.io/github/actions/workflow/status/Rubentxu/hodei-draw/deploy.yml?branch=main&label=deployment)](https://github.com/Rubentxu/hodei-draw/actions)
[![Live Demo](https://img.shields.io/badge/Live%20Demo-🚀%20Available-brightgreen)](https://rubentxu.github.io/hodei-draw/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-654FF0?logo=webassembly&logoColor=white)](https://webassembly.org/)

> **Interactive drawing and diagramming web application built with Rust/WASM**  
> An Excalidraw-style canvas with animation and physics simulation capabilities

[📖 **Read in Spanish**](README_ES.md) | [🚀 **Live Demo**](https://rubentxu.github.io/hodei-draw) | [📋 **Roadmap**](docs/ROADMAP.md)

---

## ✨ Overview

**Hodei Draw** is a next-generation interactive whiteboard built with cutting-edge web technologies. It combines the simplicity and elegance of Excalidraw with powerful animation capabilities and integrated 2D physics simulation, all running at native performance through Rust and WebAssembly.

### 🎯 Key Features

- **🎨 Intuitive Drawing Interface** - Excalidraw-inspired UI with enhanced toolbar and responsive design
- **⚡ High Performance Rendering** - WebGPU-first with Canvas2D fallback for maximum compatibility
- **🎭 Theme System** - Complete Light/Dark/System theme support with CSS custom properties
- **🔧 Multi-Shape Tools** - Rectangle, ellipse, line, and selection tools with live preview
- **📱 Responsive Design** - Works seamlessly across desktop and mobile devices
- **🌐 Modern Web Stack** - Built with Rust, WASM, Leptos, and Tailwind CSS v4

### 🚀 Planned Features (Roadmap)

- **🎬 Animation System** - Component-based animation with timeline and easing functions
- **⚛️ Physics Simulation** - Integrated 2D physics engine for interactive diagrams
- **👥 Real-time Collaboration** - P2P collaboration with rollback netcode
- **🧩 Plugin System** - Extensible architecture with WASM Component Model
- **📚 Component Libraries** - Pre-built libraries for Cloud, UML, and technical diagrams

---

## 🏗️ Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Frontend Framework** | [Leptos](https://leptos.dev) | Reactive UI with fine-grained reactivity |
| **Language** | [Rust](https://rust-lang.org) + [WebAssembly](https://webassembly.org) | High-performance, memory-safe execution |
| **Rendering** | [wgpu](https://wgpu.rs) + Canvas2D | GPU-accelerated rendering with fallback |
| **State Management** | [Bevy ECS](https://bevyengine.org/learn/book/getting-started/ecs/) | Entity Component System architecture |
| **Styling** | [Tailwind CSS v4](https://tailwindcss.com) | Utility-first CSS with custom design system |
| **Build Tool** | [Trunk](https://trunkrs.dev) | WASM-focused build tool and dev server |

---

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [Node.js](https://nodejs.org/) (18+) 
- [Trunk](https://trunkrs.dev/) - WASM build tool

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm32 target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk

# Install Node.js dependencies (for CSS processing)
cd apps/app-web && npm install
```

### Development

```bash
# Clone the repository
git clone https://github.com/Rubentxu/hodei-draw.git
cd hodei-draw

# Start development server
make serve
# OR
cd apps/app-web && trunk serve --open --features webgpu
```

The application will be available at `http://127.0.0.1:8080`

### Building for Production

```bash
# Build optimized WASM bundle
make build
# OR  
cd apps/app-web && trunk build --release
```

---

## 📁 Project Structure

```
hodei-draw/
├── 📁 apps/
│   └── 📁 app-web/          # Main web application
│       ├── 📄 src/          # Rust source code
│       ├── 📄 index.html    # HTML template
│       ├── 📄 input.css     # Tailwind CSS input
│       └── 📄 Trunk.toml    # Build configuration
├── 📁 crates/              # Rust workspace crates
│   ├── 📁 core/            # Core business logic
│   ├── 📁 ecs/             # Entity Component System
│   ├── 📁 ui-leptos/       # Leptos UI components
│   └── 📁 design-system/   # Design system & components
├── 📁 docs/                # Documentation
├── 📄 Cargo.toml          # Workspace configuration
├── 📄 Makefile            # Development commands
└── 📄 README.md           # This file
```

### Architecture

**Hodei Draw** follows a **hexagonal architecture** pattern with clear separation of concerns:

- **🎯 Core Domain** (`crates/core/`) - Business logic, independent of UI or rendering
- **⚙️ ECS Layer** (`crates/ecs/`) - Entity Component System for state management  
- **🎨 UI Layer** (`crates/ui-leptos/`) - Reactive user interface components
- **🎭 Design System** (`crates/design-system/`) - Reusable UI components and theming
- **🖼️ Rendering** (`apps/app-web/src/renderer_*`) - WebGPU and Canvas2D renderers

---

## 🛠️ Development Commands

| Command | Description |
|---------|-------------|
| `make serve` | Start development server with hot reload |
| `make build` | Build optimized production bundle |
| `make test` | Run all tests |
| `make lint` | Run clippy linter |
| `make format` | Format code with rustfmt |
| `make clean` | Clean build artifacts |

---

## 🎨 Features in Detail

### Multi-Renderer Support
- **WebGPU**: High-performance GPU rendering for modern browsers
- **Canvas2D Fallback**: Automatic fallback for broader browser compatibility
- **Runtime Switching**: Dynamic renderer selection with visual indicators

### Reactive Design System
- **Theme Support**: Light, Dark, and System themes with smooth transitions
- **Component Library**: Excalidraw-style toolbar, sidebar, and floating panels
- **Responsive Layout**: Optimized for both desktop and mobile interfaces

### Shape Tools & Preview
- **Live Preview**: Real-time shape preview during drawing operations
- **Multi-Shape Support**: Rectangle, ellipse, line tools with consistent behavior
- **Selection System**: Advanced selection tool with reactive state management

---

## 📚 Documentation

- [🏗️ **Architecture Overview**](docs/ARCHITECTURE.md) - Technical architecture and design decisions
- [🗺️ **Roadmap**](docs/ROADMAP.md) - Project roadmap and planned features  
- [📋 **Requirements**](docs/REQUISITOS.md) - Detailed functional and non-functional requirements
- [🚀 **Implementation Plan**](docs/IMPLEMENTATION_PLAN_FASES.md) - Phased development approach
- [🔌 **API Contracts**](docs/PORTS_CONTRACTS.md) - Port and adapter contracts

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Standards

- Follow [Conventional Commits](https://conventionalcommits.org/) for commit messages
- Use `cargo fmt` for code formatting
- Ensure `cargo clippy` passes without warnings
- Add tests for new functionality

---

## 🗺️ Roadmap

### Phase 1: Enhanced Drawing Foundation 🎉 **COMPLETED**
- [x] Excalidraw-style UI with modern design system
- [x] Multi-renderer architecture (WebGPU + Canvas2D)
- [x] Core shape tools with live preview
- [x] Theme system and responsive design
- [x] **Live demo public deployment** 🚀
- [x] **Professional documentation and CI/CD**
- [ ] Import/Export (SVG/PNG) - *in progress*
- [ ] Local persistence (IndexedDB) - *planned*
- [ ] Undo/Redo system - *planned*

### Phase 2: Interactive Canvas 📋 **NEXT**
- [ ] Component-based animation system
- [ ] 2D physics engine integration (Avian/Rapier)
- [ ] Timeline and keyframe management
- [ ] Advanced shape libraries

### Phase 3: Collaboration Platform 🔮 **FUTURE**
- [ ] Real-time collaboration (WebRTC + GGRS)
- [ ] Plugin system (WASM Component Model)
- [ ] Cloud synchronization
- [ ] Team workspace features

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Excalidraw](https://excalidraw.com/) - Inspiration for the user interface design
- [Bevy Engine](https://bevyengine.org/) - ECS architecture patterns
- [Leptos](https://leptos.dev/) - Reactive web framework
- [wgpu](https://wgpu.rs/) - Modern graphics API

---

<div align="center">

**[⭐ Star this repo](https://github.com/Rubentxu/hodei-draw/stargazers)** • **[🐛 Report Bug](https://github.com/Rubentxu/hodei-draw/issues)** • **[💡 Request Feature](https://github.com/Rubentxu/hodei-draw/issues)**

Made with ❤️ by [Rubentxu](https://github.com/Rubentxu)

</div>