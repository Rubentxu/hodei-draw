# Changelog

All notable changes to Hodei Draw will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-08-15

### 🎉 Initial Release - MVP Complete

**Hodei Draw v0.1.0** marks the completion of Phase 1 with a fully functional interactive drawing application. This release includes all core features for creating, editing, and manipulating shapes with professional-grade performance.

### ✨ Features Added

#### Core Drawing Functionality
- **Multi-Shape Support**: Rectangle, Ellipse, Line, and Polygon shapes with full creation and editing
- **Advanced Styling**: Fill colors, stroke colors, stroke width, and dash patterns
- **Selection System**: Precise hit-testing with visual feedback (blue border for selected shapes)
- **Drag-to-Create**: Intuitive shape creation with real-time preview

#### Rendering & Performance  
- **Dual Renderer Architecture**: WebGPU-first with automatic Canvas2D fallback
- **High Performance**: 60 FPS rendering with optimized WASM (497KB total bundle)
- **Browser Compatibility**: Automatic detection and graceful fallback for WebGPU support
- **Canvas Controls**: Runtime switching between WebGPU and Canvas2D with UI indicators

#### User Interface
- **Excalidraw-Style Design**: Modern, responsive toolbar and canvas interface
- **Design System**: Custom momentum-design-system crate with Tailwind CSS v4
- **Theme Support**: Light/Dark/System theme with smooth transitions  
- **Mobile Responsive**: Works seamlessly across desktop and mobile devices
- **Renderer Indicator**: Shows active renderer (WebGPU/Canvas2D) and device pixel ratio

#### Technical Architecture
- **Hexagonal Architecture**: Clean separation between Core, ECS, UI, and Application layers
- **Entity Component System**: Bevy ECS for efficient state management
- **Modern Web Stack**: Rust + WebAssembly, Leptos reactive framework
- **Optimized Build**: wasm-opt optimization with integrity checks

#### Developer Experience
- **Comprehensive Documentation**: Bilingual README (English/Spanish), architecture docs
- **Professional Repository**: Contributing guidelines, issue templates, MIT license
- **CI/CD Pipeline**: Automated GitHub Actions deployment with WASM optimization
- **Live Demo**: Public deployment at https://rubentxu.github.io/hodei-draw/

### 🏗️ Technical Specifications

- **Bundle Size**: 497KB optimized (HTML + CSS + WASM + JS)
- **Browser Support**: Modern browsers with WebAssembly support
- **Performance**: 60 FPS with 1000+ entities, <16ms latency
- **Architecture**: Hexagonal with clear port/adapter separation

### 📦 Package Contents

This release includes:
- Standalone web application (no server required)
- All source code with MIT license  
- Comprehensive documentation
- Build tools and development environment

### 🔧 Installation & Usage

#### Quick Start
1. Download and extract the release package
2. Serve the files with any HTTP server:
   ```bash
   # Python
   python -m http.server 8000
   
   # Node.js  
   npx serve .
   
   # Or any other HTTP server
   ```
3. Open browser to `http://localhost:8000`

#### Requirements
- Modern web browser with JavaScript enabled
- HTTP server (for serving files - browser security requirement)
- WebAssembly support (available in all modern browsers)

### 🗺️ What's Next (Phase 2)

Upcoming features in development:
- **Animation System**: Timeline-based animations with easing functions
- **2D Physics**: Integrated physics simulation for interactive diagrams  
- **Import/Export**: SVG and PNG import/export functionality
- **Local Persistence**: IndexedDB storage with autosave
- **Undo/Redo**: Complete command history with undo/redo support

### 🙏 Acknowledgments

Built with modern web technologies:
- [Rust](https://www.rust-lang.org/) + [WebAssembly](https://webassembly.org/)
- [Leptos](https://leptos.dev/) reactive framework
- [Bevy ECS](https://bevyengine.org/) for state management
- [wgpu](https://wgpu.rs/) for GPU rendering
- [Tailwind CSS v4](https://tailwindcss.com/) for styling

---

**Full Changelog**: https://github.com/Rubentxu/hodei-draw/commits/v0.1.0