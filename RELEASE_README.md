# 🎨 Hodei Draw v0.1.0 - Standalone Release

> **Interactive drawing and diagramming web application**  
> Built with Rust + WebAssembly for native performance in your browser

[🚀 **Try Live Demo**](https://rubentxu.github.io/hodei-draw) | [📋 **View Source**](https://github.com/Rubentxu/hodei-draw) | [📝 **Changelog**](CHANGELOG.md)

---

## 🚀 Quick Start

### 1. Download & Extract
Download `hodei-draw-v0.1.0.zip` and extract to any folder.

### 2. Start Local Server
The application requires an HTTP server due to browser security policies:

```bash
# Python (most systems)
python -m http.server 8000

# Python 3
python3 -m http.server 8000  

# Node.js (if installed)
npx serve . --port 8000

# PHP (if installed)  
php -S localhost:8000
```

### 3. Open in Browser
Navigate to: **http://localhost:8000**

---

## ✨ Features

### 🎨 **Core Drawing Tools**
- **Rectangle Tool** - Create and resize rectangles
- **Ellipse Tool** - Draw perfect circles and ellipses  
- **Line Tool** - Straight lines with precise endpoints
- **Selection Tool** - Select, move, and modify shapes

### ⚡ **High Performance**
- **WebGPU Rendering** - GPU-accelerated graphics when available
- **Canvas2D Fallback** - Automatic fallback for maximum compatibility
- **60 FPS Performance** - Smooth interactions with 1000+ shapes
- **Optimized WASM** - Just 497KB total bundle size

### 🎭 **Modern Interface**
- **Excalidraw-Style UI** - Clean, intuitive design
- **Theme Support** - Light, Dark, and System themes
- **Responsive Design** - Works on desktop and mobile
- **Real-time Preview** - Live shape preview while drawing

### 🔧 **Advanced Features**
- **Visual Selection** - Blue border feedback for selected shapes
- **Renderer Switching** - Runtime toggle between WebGPU/Canvas2D
- **Precise Hit Testing** - Accurate shape selection
- **Performance Indicators** - Shows active renderer and DPR

---

## 🔧 System Requirements

### **Minimum Requirements**
- Modern web browser (Chrome 57+, Firefox 53+, Safari 11+, Edge 79+)
- JavaScript enabled
- WebAssembly support (automatic in modern browsers)
- HTTP server (for local serving)

### **Recommended**
- Chrome 94+ or Firefox 89+ (for WebGPU support)
- 4GB+ RAM for complex drawings
- Hardware-accelerated graphics

### **Platform Support**
- ✅ Windows 10/11
- ✅ macOS 10.14+  
- ✅ Linux (Ubuntu 18.04+, etc.)
- ✅ iOS 11+ (Safari)
- ✅ Android 7+ (Chrome)

---

## 📁 Package Contents

```
hodei-draw-v0.1.0/
├── index.html                    # Main application
├── app-web-*.js                  # JavaScript modules  
├── app-web-*_bg.wasm            # WebAssembly binary
├── styles-*.css                  # Stylesheets
├── RELEASE_README.md             # This file
└── CHANGELOG.md                  # Release notes
```

**Total size**: ~497KB (optimized with wasm-opt)

---

## 🛠️ Development & Building

### **Building from Source**
```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install trunk
npm install  # (for CSS processing)

# Build
cd apps/app-web
trunk build --release

# Development
trunk serve --open
```

### **Project Structure** 
- **Hexagonal Architecture** with clean separation
- **Entity Component System** (Bevy ECS) for state
- **Reactive UI** with Leptos framework  
- **Dual Rendering** WebGPU + Canvas2D

---

## 🐛 Troubleshooting

### **Application Won't Load**
1. **Check HTTP Server**: Must use HTTP server, not file:// protocol
2. **Enable JavaScript**: Required for WebAssembly execution
3. **Clear Browser Cache**: Force refresh with Ctrl+F5

### **Poor Performance** 
1. **Try Canvas2D**: Use renderer toggle button in toolbar
2. **Close Other Tabs**: Free up browser memory
3. **Update Browser**: Newer versions have better WASM support

### **WebGPU Issues**
- **Chrome**: Enable `chrome://flags/#enable-unsafe-webgpu` if needed  
- **Firefox**: Set `dom.webgpu.enabled` to true in about:config
- **Fallback**: Application automatically uses Canvas2D if WebGPU unavailable

### **Mobile Issues**
- **Touch Gestures**: Use single touch for drawing, pinch to zoom
- **Performance**: Reduce canvas size for better performance
- **Keyboard**: Some mobile browsers may have input limitations

---

## 📚 Usage Guide

### **Creating Shapes**
1. Select tool from toolbar (Rectangle, Ellipse, Line)
2. Click and drag on canvas to create shape
3. Release to finish creation

### **Selecting & Editing**  
1. Choose Selection tool (arrow icon)
2. Click shapes to select (blue border indicates selection)
3. Drag to move selected shapes

### **Renderer Controls**
- **Toggle Button**: Switch between WebGPU/Canvas2D
- **Indicator**: Shows active renderer and device pixel ratio
- **Auto-fallback**: Switches to Canvas2D if WebGPU fails

---

## 🔮 Coming Soon (v0.2.0)

- **🎬 Animation System** - Timeline-based animations
- **⚛️ Physics Simulation** - 2D physics for interactive diagrams  
- **📁 Import/Export** - SVG and PNG support
- **💾 Local Storage** - Save and load projects
- **↩️ Undo/Redo** - Complete action history

---

## 📄 License & Credits

**MIT License** - Free for personal and commercial use

### **Built With**
- [Rust](https://www.rust-lang.org/) + [WebAssembly](https://webassembly.org/) - Performance & safety
- [Leptos](https://leptos.dev/) - Reactive web framework
- [wgpu](https://wgpu.rs/) - Cross-platform graphics API
- [Bevy ECS](https://bevyengine.org/) - Entity Component System
- [Tailwind CSS v4](https://tailwindcss.com/) - Utility-first styling

### **Inspired By**
- [Excalidraw](https://excalidraw.com/) - UI/UX design patterns
- Modern web standards and best practices

---

## 🆘 Support

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/Rubentxu/hodei-draw/issues)
- **💡 Feature Requests**: [GitHub Discussions](https://github.com/Rubentxu/hodei-draw/discussions)  
- **📖 Documentation**: [Project Wiki](https://github.com/Rubentxu/hodei-draw/wiki)
- **🚀 Live Demo**: [Try Online](https://rubentxu.github.io/hodei-draw)

---

<div align="center">

**Hodei Draw v0.1.0** - Built with ❤️ using Rust + WebAssembly

[⭐ Star on GitHub](https://github.com/Rubentxu/hodei-draw) • [🚀 Try Live Demo](https://rubentxu.github.io/hodei-draw)

</div>