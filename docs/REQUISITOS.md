Resumen corto de requisitos

FR-Core: lienzo infinito, zoom/pan, formas básicas, texto, lápiz libre, importar imagen/SVG, exportar PNG/SVG, guardado/carga JSON.
FR-Animación: animar Transform y Style, easing múltiples, timeline básica, controles Play/Pause/Reset.
FR-Física: modo simulación, RigidBody, Collider, ajustes globales, joints.
NFR: 60 FPS, WASM optimizado, compat. Chrome/Firefox/Safari, cifrado E2E en colaboración, UX simple.
Stack: Rust+WASM, Leptos (UI), bevy_ecs (ECS), wgpu+lyon+glyphon (render vector y texto), avian/bevy_rapier (física), matchbox+ggrs (colaboración), rexie/IndexedDB (local-first), objetivo futuro: WASM Component Model (plugins).
Arquitectura hexagonal (puertos y adaptadores) Capas:

Dominio (core)
Modelos y lógica pura del lienzo: EntityId, Shape, Style, Transform, Document, Selection, Commands, Constraints.
Reglas de negocio para dibujo, edición, timeline de animación, parámetros de simulación.
Puertos (traits) para persistencia, render, input, tiempo/reloj, física, colaboración.
Aplicación (casos de uso)
Orquestación de casos de uso: CreateShape, UpdateTransform, Group/Ungroup, Import/Export, Start/StopSimulation, AddAnimation, PersistProject.
Servicios de aplicación: CommandBus/UndoRedo, Transaction/History, Validation, Serialization.
Adaptadores (entradas/salidas)
UI/Web (Leptos): componentes y señales que llaman casos de uso.
Render (wgpu/lyon/glyphon): implementación del puerto RenderPort.
Física (avian/rapier): implementación de PhysicsPort.
Persistencia (rexie/IndexedDB): StoragePort.
Colaboración (matchbox/ggrs): SyncPort/NetPort.
Plataforma (WASM bindings, timers, fs virtual, clipboard): SystemPorts.
ECS y separación de responsabilidades

ECS para estado del lienzo (entidades y componentes): Transform, Drawable(Shape), Style, Text, Image, Physics(RigidBody/Collider), AnimationTrack, SelectionTag, ZIndex.
Dominio define componentes y sistemas “puros” donde sea posible; adaptadores invocan sistemas dependientes de infraestructura (render/physics).
Leptos gestiona UI reactiva; comunicación con capa aplicación mediante comandos/eventos; sincronización UI↔ECS por adaptadores de proyección/queries.
Estructura de workspace (multi-crate)

hodei-draw/
Cargo.toml (workspace)
crates/
core/ (dominio puro)
src/
lib.rs
entity.rs (EntityId, uuids)
components/
transform.rs, style.rs, shape.rs, text.rs, image.rs, physics.rs, animation.rs, metadata.rs
systems/
editing.rs (create/move/resize), selection.rs, grouping.rs
animation.rs (sampling, easing), timeline.rs
serialization.rs (schema JSON)
ports/
render.rs (RenderPort), storage.rs (StoragePort), physics.rs (PhysicsPort), time.rs (TimePort)
collab.rs (SyncPort), asset.rs (AssetPort)
usecases/
draw.rs, edit.rs, import_export.rs, persist.rs, simulate.rs, animate.rs
services/
command_bus.rs, undo_redo.rs, history.rs, validation.rs
ecs/ (configuración bevy_ecs standalone)
src/ (world setup, schedulers, resources, events, adapters a core)
adapters-render-wgpu/
src/ (pipeline wgpu, teselación lyon, texto glyphon; implementa RenderPort)
adapters-physics-avian/
src/ (mapeo componentes core<->avian; implementa PhysicsPort)
adapters-storage-indexeddb/
src/ (rexie; implementa StoragePort)
adapters-collab-webrtc/
src/ (matchbox+ggrs; implementa SyncPort)
adapters-platform-wasm/
src/ (bindings wasm-bindgen/wasm-bindgen-futures, timers, files virtuales)
ui-leptos/
src/
app.rs (root)
components/
toolbar.rs, sidebar.rs, layers_panel.rs, timeline.rs, properties_panel.rs
canvas_view.rs (integra con render y ECS mediante proyección)
state/
signals.rs (selección, herramienta activa), commands.rs (acciones hacia casos de uso)
routes/ (si aplica)
app-web/ (binario WASM para navegador)
src/main.rs (configura UI Leptos, world ECS, registra adaptadores)
index.html, styles, assets
app-desktop/ (opcional fase 3)
src/main.rs (winit/tauri)
scripts/ (build, wasm-opt, wasm-bindgen, trunk/wasm-pack)
docs/ (ADR, diagramas)
tests/ (integración e2e con wasm-bindgen-test, UI tests básicos)
Puertos clave (traits) en core

RenderPort: begin_frame/end_frame, draw_shape(shape tess), draw_text, measure_text, hit_test opcional.
PhysicsPort: step(dt), add/remove bodies, update shapes↔colliders, query contacts.
StoragePort: save_project(doc_id, bytes), load_project(doc_id), list_projects.
SyncPort: broadcast(op), receive() -> stream de ops; session management.
TimePort: now(), request_animation_frame(callback) en WASM via adaptador.
AssetPort: cargar imágenes y fuentes.
Flow de datos y eventos

UI (Leptos) emite comandos de usuario → Casos de uso en aplicación (core) → mutan ECS/estado (core) → eventos → Adaptador Render repinta → UI observa señales derivadas (proyección de estado no-GPU).
Animación: timeline (core) avanza con TimePort; systems de sampling aplican interpolaciones a componentes; RenderPort refleja el estado.
Simulación: PhysicsPort step sincroniza con componentes Physics; sistemas sincronizan Transform<->RigidBody tras step.
Formatos y serialización

Documento JSON abierto: proyecto, entidades, componentes; versiones con semver y migraciones.
Export: PNG via render offscreen y captura; SVG via lyon paths + estilos; Import: SVG parseado a Shapes.
Estrategia de testing

Unit tests en core (sistemas y casos de uso).
Property-based tests en operaciones de edición/undo-redo.
Snapshot tests para serialización JSON y export SVG.
Integración: wasm-bindgen-test para UI mínima y ciclos de animación/simulación.
Performance benches en nativa (criterion) para sistemas críticos.
Build y herramientas

Targets: wasm32-unknown-unknown (web), x86_64-unknown-… (desktop futuro).
Bundling: trunk o wasm-pack + Vite (elige uno; recomendación: Trunk por Leptos).
Optimización: wasm-bindgen, wasm-opt -O, code splitting si aplica.
Lint/format: rustfmt, clippy, cargo-deny; pre-commit.
CI: build+test, wasm size budget, e2e mínimas de render.
Plan por fases (entregables claros)

Fase 1: MVP (El Mejor Excalidraw)
Crates: core, ecs, adapters-render-wgpu (mínimo shapes, texto), adapters-platform-wasm, ui-leptos, adapters-storage-indexeddb, app-web.
Funciones: formas básicas, texto, lápiz, zoom/pan, import SVG básico, export PNG/SVG, guardar/cargar JSON local.
Rendimiento: 60 FPS con 1k entidades simples; selección y manipulación fluidas; undo/redo estable.
Entregable: demo web pública.
Fase 2: Lienzo Interactivo
Crates: adapters-physics-avian, animación completa en core (timeline, easing), integración TimePort, controles de reproducción.
Colaboración básica: matchbox+ggrs para sincronizar comandos con rollback netcode en proyectos pequeños.
Entregable: demos de animación y simulación con ejemplos (gravedad, colisiones, juntas simples).
Fase 3: Plataforma Abierta
Librerías de componentes (UML, Cloud).
Iniciar arquitectura de plugins (WASM Component Model): definir límites de sandbox y API estable de puertos.
Desktop packaging (opcional) y features Plus/Enterprise.
Decisiones y riesgos

wgpu en Web: usar backend WebGL2 como fallback si WebGPU no está disponible; abstraer en RenderPort.
Texto: glyphon o swash; validar soporte de fuentes en WASM.
Física: empezar con avian; si bloquea, fallback a bevy_rapier con capa de compatibilidad.
Sincronización P2P: ggrs requiere determinismo; aislar cualquier fuente de no-determinismo (floating point) y aplicar fixed timestep.
Tamaño WASM: objetivos de <3–5 MB gz; controlar features y LTO.
Primeros pasos concretos

Inicializar workspace y crates vacíos según estructura anterior (sin implementar lógica).
Definir traits de puertos en core/ports y modelos base (EntityId, Shape, Transform, Style).
Montar ui-leptos con layout mínimo y canvas_view que llame a RenderPort (stub).
Integrar adapters-platform-wasm con requestAnimationFrame y bucle de render.
Implementar sistemas mínimos de creación/selección/movimiento en core y su wiring desde UI.
Persistencia mínima con IndexedDB (save/load proyecto).
Benchmarks básicos de rendimiento en edición y render inicial.