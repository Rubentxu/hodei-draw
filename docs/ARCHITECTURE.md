# 🏗️ Hodei Draw Architecture

This document provides an overview of the technical architecture and design decisions for the Hodei Draw project.

## 🎯 Architecture Overview

Hodei Draw follows a **Hexagonal Architecture** (also known as Ports and Adapters) pattern, ensuring clean separation of concerns and high testability.

```
┌─────────────────────────────────────────────────────────┐
│                    External World                        │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐ │
│  │   Browser   │  │   WebGPU     │  │   IndexedDB     │ │
│  │   Events    │  │   Canvas2D   │  │   LocalStorage  │ │
│  └─────────────┘  └──────────────┘  └─────────────────┘ │
└─────────────────────────┬───────────────────────────────┘
                          │
         ┌────────────────┴────────────────┐
         │           Adapters              │
         │  ┌─────────────────────────────┐ │
         │  │      ui-leptos crate        │ │
         │  │   (Leptos Components)       │ │
         │  └─────────────────────────────┘ │
         │  ┌─────────────────────────────┐ │
         │  │   design-system crate       │ │
         │  │  (UI Components & Themes)   │ │
         │  └─────────────────────────────┘ │
         └────────────────┬────────────────┘
                          │
         ┌────────────────┴────────────────┐
         │            Ports                │
         │  ┌─────────────────────────────┐ │
         │  │        ecs crate            │ │
         │  │   (State Management)        │ │
         │  └─────────────────────────────┘ │
         └────────────────┬────────────────┘
                          │
         ┌────────────────┴────────────────┐
         │          Core Domain            │
         │  ┌─────────────────────────────┐ │
         │  │        core crate           │ │
         │  │    (Business Logic)         │ │
         │  └─────────────────────────────┘ │
         └─────────────────────────────────┘
```

## 📦 Crate Structure

### Core Domain (`crates/core/`)
- **Purpose**: Pure business logic, independent of any framework or external dependency
- **Dependencies**: None (except for basic utilities like `serde`)
- **Responsibilities**:
  - Domain entities (Shape, Canvas, Tool)
  - Business rules and validation
  - Pure functions for calculations
  - Domain events and commands

### ECS Layer (`crates/ecs/`)
- **Purpose**: State management using Entity Component System pattern
- **Dependencies**: `bevy_ecs`, `core`
- **Responsibilities**:
  - Entity definitions (Canvas entities, Shape entities)
  - Component definitions (Transform, Style, Physics)
  - System implementations (rendering pipeline, input handling)
  - Resource management (global state, configuration)

### UI Adapter (`crates/ui-leptos/`)
- **Purpose**: User interface implementation using Leptos framework
- **Dependencies**: `leptos`, `ecs`, `design-system`
- **Responsibilities**:
  - React to state changes from ECS
  - Handle user input and convert to ECS events
  - Coordinate between UI components and business logic
  - WebAssembly bindings and browser APIs

### Design System (`crates/design-system/`)
- **Purpose**: Reusable UI components and theming system
- **Dependencies**: `leptos`, `serde` (for theme serialization)
- **Responsibilities**:
  - UI component library (buttons, toolbars, panels)
  - Theme management (light/dark/system themes)
  - Icon system and typography
  - CSS custom properties and styling utilities

### Application (`apps/app-web/`)
- **Purpose**: Main application entry point and rendering backends
- **Dependencies**: All crates, `wgpu`, `web-sys`
- **Responsibilities**:
  - Application bootstrap and initialization
  - Rendering backend implementations (WebGPU, Canvas2D)
  - Asset loading and resource management
  - Platform-specific integrations

## 🎨 Rendering Architecture

### Multi-Backend Rendering System

```rust
trait RenderPort {
    fn clear_canvas(&mut self, color: Color);
    fn draw_shape(&mut self, shape: &Shape, transform: &Transform);
    fn present(&mut self);
}

struct WebGPURenderer { /* ... */ }
struct Canvas2DRenderer { /* ... */ }

impl RenderPort for WebGPURenderer { /* ... */ }
impl RenderPort for Canvas2DRenderer { /* ... */ }
```

### Rendering Pipeline

1. **ECS Systems** generate rendering commands
2. **Render Commands** are collected in a command buffer
3. **Backend Selection** chooses WebGPU or Canvas2D
4. **Command Execution** renders to the selected backend
5. **Present** displays the final result

### Fallback Strategy

```
WebGPU Available? ──Yes──> Use WebGPU Renderer
     │
     No
     │
     └──> Use Canvas2D Renderer (Always Available)
```

## 🔄 State Management with ECS

### Entity-Component-System Pattern

```rust
// Entities: Unique IDs for canvas objects
let shape_entity = world.spawn().id();

// Components: Data containers
#[derive(Component)]
struct Transform {
    position: Vec2,
    rotation: f32,
    scale: Vec2,
}

#[derive(Component)]
struct Style {
    fill_color: Color,
    stroke_color: Color,
    stroke_width: f32,
}

// Advanced Hit Testing: Separates interaction zones from visual shapes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Hitbox {
    FromShape { tolerance: f32 },    // Use shape geometry + tolerance
    Rect { x: f32, y: f32, w: f32, h: f32 },  // Custom rectangular area
    Circle { x: f32, y: f32, radius: f32 },   // Custom circular area
    Polygon { points: Vec<(f32, f32)> },      // Custom polygonal area
    Multiple(Vec<Hitbox>),           // Composite hitboxes
    None,                            // No interaction possible
}

// Systems: Logic that operates on components
fn render_system(
    query: Query<(&Transform, &Style), With<Shape>>,
    mut renderer: ResMut<Box<dyn RenderPort>>,
) {
    for (transform, style) in query.iter() {
        renderer.draw_shape_with_style(transform, style);
    }
}
```

### Data Flow

```
User Input → Leptos Event → ECS Command → System Processing → Component Updates → Render System → Display
```

## 🎭 Theme System Architecture

### CSS Custom Properties Strategy

```css
:root {
  --island-bg-color: #ffffff;
  --ui-text-color: #495057;
  --button-selected-bg: #007bff;
}

.theme-dark {
  --island-bg-color: #232329;
  --ui-text-color: #ffffff;
  --button-selected-bg: #007bff;
}
```

### Theme Provider Pattern

```rust
#[derive(Clone)]
pub struct ThemeProvider {
    theme: RwSignal<Theme>,
    system_theme: RwSignal<Theme>,
}

impl ThemeProvider {
    pub fn set_theme(&self, theme: Theme) {
        self.theme.set(theme);
        self.apply_theme();
    }
    
    fn apply_theme(&self) {
        // Update HTML class names
        // Persist to localStorage
    }
}
```

## 🚀 Performance Optimizations

### WASM Optimization Strategy

1. **Rust Compiler Optimizations**
   ```toml
   [profile.release]
   opt-level = "s"  # Optimize for size
   lto = true       # Link-time optimization
   codegen-units = 1
   ```

2. **WASM-pack Optimizations**
   ```bash
   wasm-opt -Oz -o optimized.wasm input.wasm
   ```

3. **Bundle Splitting**
   - Core application bundle
   - Lazy-loaded feature modules
   - Asset streaming

### Rendering Performance

1. **Command Batching**: Group similar render operations
2. **Dirty Flagging**: Only redraw changed regions
3. **GPU Instancing**: Render multiple similar objects efficiently
4. **Level-of-Detail**: Reduce complexity at high zoom-out levels

### Memory Management

1. **Object Pooling**: Reuse allocations for temporary objects
2. **Component Packing**: Efficient data layout for cache performance
3. **Sparse Sets**: Efficient component storage in ECS
4. **WASM Linear Memory**: Careful management of WebAssembly heap

## 🔌 Extension Points

### Plugin Architecture (Future)

```rust
#[wasm_bindgen]
pub trait Plugin {
    fn initialize(&self, world: &mut World);
    fn update(&self, world: &mut World);
    fn cleanup(&self);
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}
```

### Custom Renderers

```rust
pub trait CustomRenderer: RenderPort {
    fn supports_feature(&self, feature: RenderFeature) -> bool;
    fn get_priority(&self) -> u32;
}
```

### Component Extensions

```rust
// Custom components can be added to extend functionality
#[derive(Component)]
struct PhysicsBody {
    mass: f32,
    velocity: Vec2,
    friction: f32,
}

// Custom systems process these components
fn physics_system(mut query: Query<(&mut Transform, &mut PhysicsBody)>) {
    // Physics simulation logic
}
```

## 🧪 Testing Strategy

### Unit Testing
- **Core Logic**: Pure functions with property-based testing
- **Components**: Isolated component behavior testing  
- **Systems**: Mock dependencies and verify system behavior

### Integration Testing
- **ECS Integration**: Test component and system interactions
- **Rendering**: Test render command generation and execution
- **UI Events**: Test user interaction flows

### End-to-End Testing
- **Browser Automation**: Puppeteer/Playwright for full user workflows
- **Visual Regression**: Screenshot comparisons for UI consistency
- **Performance Testing**: Benchmark rendering and interaction performance

## 🔒 Security Considerations

### WASM Security Model
- **Sandboxed Execution**: WASM runs in browser security sandbox
- **Memory Safety**: Rust prevents buffer overflows and memory corruption
- **API Surface**: Limited to explicitly imported browser APIs

### Data Security
- **Local-First**: Data stored locally by default
- **Serialization**: Secure serialization with `serde`
- **Input Validation**: All user input validated at domain boundaries

## 📈 Scalability Considerations

### Performance Scaling
- **Virtual Canvas**: Only render visible regions
- **Spatial Indexing**: Efficient spatial queries for large diagrams
- **Web Workers**: Offload heavy computation to background threads

### Code Scaling  
- **Modular Architecture**: Clean boundaries between modules
- **Plugin System**: Third-party extensions without core modifications
- **Feature Flags**: Conditional compilation for different feature sets

## 🔄 Future Evolution

### Phase 2: Animation System
```rust
#[derive(Component)]
struct AnimationTarget {
    property: AnimatedProperty,
    keyframes: Vec<Keyframe>,
    duration: Duration,
}

fn animation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &AnimationTarget)>
) {
    // Animation interpolation logic
}
```

### Phase 3: Physics Integration
```rust
#[derive(Component)]
struct RigidBody {
    body_type: BodyType,
    mass: f32,
    friction: f32,
}

fn physics_system(
    mut physics_world: ResMut<PhysicsWorld>,
    query: Query<(Entity, &Transform, &RigidBody)>
) {
    // Physics simulation step
}
```

### Phase 4: Collaboration
```rust
#[derive(Event)]
struct NetworkEvent {
    peer_id: PeerId,
    operation: Operation,
    timestamp: u64,
}

fn sync_system(
    mut events: EventReader<NetworkEvent>,
    mut world_state: ResMut<WorldState>
) {
    // Operational transform and conflict resolution
}
```

---

This architecture provides a solid foundation for building a high-performance, extensible drawing application while maintaining clean separation of concerns and excellent testability.