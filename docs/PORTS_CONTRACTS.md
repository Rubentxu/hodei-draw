# Contratos de Puertos (Ports) — Hodei Momentum

Versión: 1.0
Fecha: 2025-08-14

Propósito
Este documento define los contratos (traits) del dominio que establecen las interfaces entre el núcleo (core) y las implementaciones externas (adaptadores). Los puertos viven en el crate `core` y deben ser totalmente agnósticos a la plataforma. Todas las implementaciones específicas (Web/WASM) se encuentran en adaptadores dentro de `apps/app-web` u otros crates de infraestructura.

Principios
- Core sin dependencias de plataforma (sin wasm-bindgen/web-sys).
- Errores tipados con thiserror en core. En app usar anyhow si conviene.
- Los puertos modelan capacidades mínimas necesarias; dejar espacio a extensiones via feature flags o traits adicionales.
- Rendimiento y batching: las operaciones se agrupan entre `begin_frame`/`end_frame`.

Tipos base (core)
- Vec2(f32, f32), Color(R,G,B,A f32), Mat3/Mat4 opcionales, Rect {x,y,w,h}, Path (lista de comandos), TextSpan {text, style}.
- Ids fuertes: DocumentId, LayerId, ShapeId.
- Document/Shape/Style/Transform definidos en core.

RendererPort
Responsable de dibujar formas vectoriales y texto, bajo un modelo de frame explícito.

- begin_frame(viewport: Viewport) -> Result<(), RenderError>
  - Señala el inicio de un frame. Debe limpiar/ajustar estados necesarios.
- end_frame() -> Result<(), RenderError>
  - Flushea comandos y presenta el frame.
- set_camera(transform: Mat3) -> Result<(), RenderError>
  - Define pan/zoom/rotación del lienzo.
- draw_shape(shape: &Shape, style: &Style, world: &Transform) -> Result<(), RenderError>
  - Dibuja una forma básica (Rect, Ellipse, Line, Arrow, Path); world compone la transform.
- draw_path(path: &Path, style: &Style, world: &Transform) -> Result<(), RenderError>
  - Alternativa explícita cuando el shape ya está resuelto como Path.
- draw_text(span: &TextSpan, origin: Vec2, world: &Transform) -> Result<(), RenderError>
  - Render de texto en una posición; admite estilos básicos (font, size, weight, fill).
- measure_text(span: &TextSpan) -> Result<TextMetrics, RenderError>
  - Métrica de layout (width, ascent, descent, line_gap). Determinista para layout estable.
- upload_image(id: ImageId, data: ImageData) -> Result<(), RenderError>
  - Carga/actualiza una textura en caché del renderer; usada para imágenes importadas.
- draw_image(id: ImageId, dest: Rect, world: &Transform, tint: Option<Color>) -> Result<(), RenderError>
  - Dibuja imagen referenciada por id.

Notas de implementación (Web)
- Adaptador WGPU: crear pipelines para fill/stroke; teselación con lyon; atlas de texto vía glyphon; cache de fonts.
- Fallback Canvas2D implementado como adaptador de `RenderPort` para asegurar salida visual cuando WebGPU no está disponible. Incluye `begin_frame` con limpieza y `draw_shape` para rectángulos básicos.
- Fallback WebGL2 via wgpu (plan futuro) para cobertura más amplia de navegadores.
- Evitar asignaciones por frame; usar buffers persistentes y batching.

RenderError (core)
- Variantes: Initialization, DeviceLost, SurfaceLost, OutOfMemory, InvalidInput, Unsupported, TextShaping, UploadFailed, Other(String).

StoragePort
Responsable de persistencia local-first. En Web se implementa con IndexedDB (rexie).

- save_project(project: Project) -> Result<ProjectId, StorageError>
  - Crea o actualiza un proyecto (con schema_version). Debe ser transaccional.
- load_project(id: ProjectId) -> Result<Project, StorageError>
  - Lee un proyecto por id.
- list_projects() -> Result<Vec<ProjectMeta>, StorageError>
  - Lista proyectos recientes con metadatos (id, name, updated_at, size).
- delete_project(id: ProjectId) -> Result<(), StorageError>
  - Borra un proyecto y blobs asociados.
- put_blob(id: BlobId, bytes: Bytes) -> Result<(), StorageError>
  - Guarda binarios asociados (imágenes, thumbnails).
- get_blob(id: BlobId) -> Result<Option<Bytes>, StorageError>
  - Recupera un blob si existe.
- migrate(from: u32, to: u32) -> Result<(), StorageError>
  - Ejecuta migraciones de esquema. No debe perder datos.

Modelos (core)
- Project { id: ProjectId, name: String, document: Document, schema_version: u32, thumbnails: Vec<BlobId>, updated_at: Timestamp }
- ProjectMeta { id, name, updated_at, size }
- Timestamp: epoch millis (u64) o chrono simple sin zonas.

StorageError (core)
- Variantes: NotFound, Conflict, QuotaExceeded, Serialization, Deserialization, Backend(String), Unsupported, Other(String).

ClipboardPort (opcional F1)
- copy(selection: Selection) -> Result<(), ClipboardError>
- paste() -> Result<Option<DocumentFragment>, ClipboardError>
- Compatibilidad Web: usar Clipboard API a través de adaptador con serialización JSON + PNG fallback.

Contratos de concurrencia y temporización
- ECS actualiza estado en fixed timestep (p.ej. 60 Hz). RenderPort opera en cada rAF.
- El adaptador de renderer debe ser thread-safe en nativo; en wasm single-threaded, mantener invariantes con RefCell cuidadosamente encapsulado.

Determinismo
- measure_text debe ser estable entre sesiones para reproducibilidad.
- Importante para Fase 2 (rollback netcode): evitar fuentes no deterministas; definir un seed global para cualquier aleatoriedad.

Gating WASM y no-ops
- Core expone traits y tipos. Proveer implementaciones no-op para targets no wasm32 (útil para tests nativos).
- Usar cfg_if en adaptadores para seleccionar implementaciones.

Versionado y extensibilidad
- Evolucionar traits con métodos opcionales detrás de feature flags.
- Mantener cambios breaking agrupados entre fases mayores; documentar migraciones.

Ejemplo de flujo por frame (pseudocódigo)
- renderer.begin_frame(viewport)
- renderer.set_camera(camera)
- for entity in visible_entities { renderer.draw_shape(&entity.shape, &entity.style, &entity.transform) }
- for text in visible_text { renderer.draw_text(&text.span, text.origin, &text.transform) }
- renderer.end_frame()
