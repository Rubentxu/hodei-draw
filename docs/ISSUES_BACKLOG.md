# Backlog de Issues — Fase 1 (MVP)

Convenciones
- ConvCommits: feat/fix/refactor/docs/test/chore/style/perf
- Labels sugeridas: area:core, area:ecs, area:ui, area:render, area:storage, type:infra, type:feature, type:bug, perf, size:S/M/L, prio:P1/P2/P3
- Definición de Hecho (DoD): compilación workspace, clippy sin warnings, tests relevantes, docs actualizadas

Listado (orden recomendado de PRs pequeños)

1) chore: workspace y crates base
- Descripción: Configurar workspace Cargo con crates core, ecs, ui-leptos y app-web. Makefile de tareas.
- Criterios de aceptación:
  - Comando `cargo metadata` muestra workspace; `make serve/build/test/lint` disponible
  - `core` no depende de wasm-bindgen ni web-sys
- Labels: type:infra, prio:P1, size:S

2) feat(core): puertos Renderer y Storage
- Descripción: Definir traits RendererPort y StoragePort (y errores con thiserror). Implementaciones no-op para target no-wasm.
- Criterios:
  - Traits en `core` con docs; errores tipados; `cfg` notas
  - Tests unitarios de contratos mínimos (mock impl)
- Labels: area:core, type:feature, prio:P1, size:M

3) feat(ecs): recursos y eventos mínimos
- Descripción: Definir recursos Document, Selection, Viewport, ToolState; eventos de puntero y creación de formas.
- Criterios:
  - Sistemas compilan y no dependen de web-sys; pruebas unitarias simples
- Labels: area:ecs, type:feature, prio:P1, size:M

4) feat(app-web): bootstrap wgpu + loop RAF
- Descripción: Inicializar wgpu (con fallback) y establecer bucle RAF; conectar a adapter RendererPort (stub).
- Criterios:
  - Pantalla de color claro 60 FPS; logs con tracing-wasm; sin pánicos
- Labels: area:render, type:feature, prio:P1, size:M

5) feat(ui): shell Leptos y canvas host
- Descripción: Crear estructura UI (toolbar, panel propiedades, canvas host). Señales básicas y envío de eventos.
- Criterios:
  - Interacción de botones cambia herramienta; eventos puntero llegan al ECS
- Labels: area:ui, type:feature, prio:P1, size:M

6) feat(ecs): crear/seleccionar/mover Rect/Line
- Descripción: Sistemas para crear rectángulos y líneas, selección con clic/drag, mover con arrastre.
- Criterios:
  - Demo permite añadir y mover figuras; selección visible
- Labels: area:ecs, type:feature, prio:P1, size:M

7) feat(render): teselación lyon + estilos
- Descripción: Teselar formas vectoriales, batch básico, soporte de stroke/fill, cámara pan/zoom.
- Criterios:
  - 60 FPS con 1k rectángulos simples; zoom/pan suaves
- Labels: area:render, type:feature, perf, prio:P1, size:L

8) feat(text): glyphon + measure_text
- Descripción: Integrar texto con glyphon; implementar measure_text en port.
- Criterios:
  - Render de texto nítido; medición usada por UI para layout
- Labels: area:render, area:ui, type:feature, prio:P2, size:M

9) feat(ecs): scale/rotate; handles; pencil
- Descripción: Manipulación avanzada con handles; lápiz con simplificación (RDP).
- Criterios:
  - Escala/rotación fluidas; línea a mano alzada suavizada
- Labels: area:ecs, type:feature, prio:P2, size:L

10) feat(import/export): SVG in; PNG/SVG out
- Descripción: Importar SVG básico; exportar selección/vista a SVG y PNG (readback).
- Criterios:
  - Casos simples de SVG importan bien; export produce archivos correctos
- Labels: area:core, area:render, area:ui, type:feature, prio:P2, size:M

11) feat(storage): IndexedDB rexie; autosave; recientes
- Descripción: Implementar adapter StoragePort (web) con schema versionado.
- Criterios:
  - Guardar/cargar documento; autosave estable; lista de recientes
- Labels: area:storage, type:feature, prio:P1, size:M

12) feat(core/ecs): undo/redo y tests
- Descripción: Pila de comandos con apply/revert; tests de snapshot y property-based.
- Criterios:
  - Undo/redo para crear/mover/editar; cobertura >80% en core crítico
- Labels: area:core, area:ecs, type:feature, prio:P1, size:M

13) perf: optimización WASM y size budget
- Descripción: wasm-opt, LTO, opt-level=z; revisar dependencias; medir con tracing.
- Criterios:
  - WASM gzip <3–5 MB; 60 FPS con 1k entidades
- Labels: perf, type:infra, prio:P1, size:M

14) docs/chore: demo pública y documentación
- Descripción: Publicar demo; actualizar README y docs; changelog inicial.
- Criterios:
  - Demo accesible; documentación alineada; checklist DoD pasado
- Labels: docs, type:infra, prio:P2, size:S

Dependencias entre issues (resumen)
- (2) depende de (1)
- (3) depende de (2)
- (4) depende de (1)
- (5) depende de (3) y (4)
- (6) depende de (3)
- (7) depende de (4)
- (8) depende de (7)
- (9) depende de (6)
- (10) depende de (7)
- (11) depende de (2) y (1)
- (12) depende de (3)
- (13) depende de (7,8,11)
- (14) depende de MVP completo
