# Plan de Implementación — Hodei Momentum

Versión: 1.0
Fecha: 2025-08-14

Este documento concreta el plan de ejecución por fases alineado con:
- PRD (docs/PRD.md)
- Roadmap (docs/ROADMAP.md)
- Guía de arquitectura y normas (hodei.md)

Principios rectores
- Arquitectura hexagonal estricta: core (dominio, puertos) → ecs (orquestación) → ui/adaptadores (driving/driven)
- WebAssembly: código específico bajo cfg wasm32; núcleo multiplataforma
- Performance: 60 FPS, <16 ms por frame; tamaño WASM objetivo <3–5 MB gzip
- Local-first: persistencia en IndexedDB vía rexie
- Clean code: clippy sin warnings, tests y snapshots en core crítico

Progreso reciente (estado a 2025-08-14)
- Fallback automático a Canvas2D cuando WebGPU no está disponible o falla la inicialización (detección previa de `navigator.gpu`), sin ruido en consola.
- Controles de UI para conmutar Canvas2D/WebGPU en caliente. El botón WebGPU se deshabilita si no hay soporte (tooltip).
- Indicador en la UI del renderer activo y del DPR, reactivo a cambios de renderer y de `resize`.

Pila tecnológica
- UI: Leptos (CSR)
- ECS: bevy_ecs (standalone)
- Render GPU: wgpu + lyon (vectorial) + glyphon (texto)
- Persistencia: IndexedDB (rexie)
- Animación (F2): bevy_tweening o equivalente
- Física (F2): avian (preferido) o bevy_rapier
- Colaboración (F2): matchbox (WebRTC) + ggrs (rollback netcode)

Fase 1 — “El Mejor Excalidraw” (MVP)
Objetivo: base sólida de edición y render con persistencia local.

Épicas
1) Infra/workspace y puertos
- Crates: core, ecs, ui-leptos, apps/app-web
- Puertos en core: RendererPort, StoragePort (ClipboardPort opcional)
- Adaptadores web en app-web (driven) y UI Leptos (driving)

2) Modelo de dominio (core)
- Tipos: Document, Layer, Shape (Rect/Ellipse/Line/Arrow/Path), TextRun, Style, Transform
- IDs fuertes (newtype): DocumentId, LayerId, ShapeId
- Serialización JSON (serde) con schema_version; thiserror para errores

3) Orquestación ECS (ecs)
- Recursos: Document, Selection, ToolState, Viewport, UndoRedoStack
- Eventos: PointerDown/Move/Up, KeyDown, ShapeCreated/Updated, SelectionChanged
- Sistemas: creación/selección/manipulación, freehand con simplificación

4) Render vectorial (app-web + adapter Renderer)
- Inicialización wgpu (fallback WebGL2 mediante wgpu) y fallback a Canvas2D cuando WebGPU no esté disponible (implementado)
- Teselación con lyon, batch y estados; cámara con pan/zoom
- Texto con glyphon; caché de fuentes; measure_text vía puerto

5) UI Leptos (ui-leptos)
- Shell: toolbar, panel propiedades, canvas host
- Señales de estado (herramienta/color/estilo) y puente a ECS
- Manejo de eventos de puntero/teclado → eventos ECS
- Controles para conmutar Canvas2D/WebGPU y mostrar indicador `Renderer: <Nombre> | DPR: <valor>` (implementado)

6) Import/Export
- Importar SVG básico (rect/circle/line/path)
- Exportar selección/vista a SVG y PNG (render a textura + bridge canvas)

7) Persistencia local
- StoragePort (web): rexie con stores versionadas; autosave; lista proyectos

8) Undo/Redo y calidad
- Comandos apply/revert con coalescing; límites de stack
- Tests: snapshot en core y property-based donde aplique

9) Performance WASM
- Trunk + wasm-opt; LTO y opt-level=z; tracing-wasm para medir

Timeline sugerido (10 semanas)
- S1-2: workspace, puertos core, UI mínima, loop RAF
- S3-4: render wgpu+lyon, texto básico, pan/zoom; fallback Canvas2D cuando WebGPU no esté disponible; controles de conmutación e indicador renderer/DPR
- S5-6: selección/crear/mover/escala/rotar, pencil
- S7: import/export SVG/PNG
- S8: IndexedDB + JSON versionado + autosave
- S9: undo/redo + tests snapshot/property
- S10: optimización WASM + demo pública MVP

Criterios de aceptación (F1)
- 60 FPS con 1k entidades simples; latencia <16 ms
- WASM gzip <3–5 MB en Release
- Clippy sin warnings; >80% cobertura en core crítico
- Smoke test UX: crear/editar formas y texto en <2 min por usuario nuevo
- Estabilidad: 0 crashes en 30 min de pruebas
- UX de compatibilidad: en navegadores sin WebGPU, la app debe iniciar sin errores, con Canvas2D activo, botón WebGPU deshabilitado y indicador mostrando correctamente el backend y el DPR.

Riesgos y mitigaciones (F1)
- WebGPU no disponible → fallback Canvas2D (implementado) y, a futuro, WebGL2 (wgpu). Señalización clara en UI y controles de conmutación.
- Texto en WASM → validar glyphon/swash temprano; caché de fonts
- Tamaño WASM → LTO, opt-level=z, feature flags, auditoría de dependencias

Fase 2 — Lienzo Interactivo
- Animación/tweening: easings sobre Transform/Style; timeline; UI playback
- Física 2D: avian; RigidBody, Collider, joints básicas; modo Simulación
- Colaboración P2P: matchbox + ggrs; rollback sobre comandos; determinismo
- Demos públicas: animación de flujo y simulación de colisiones/gravedad

Fase 3 — Plataforma Abierta
- Librerías de componentes (UML/Cloud)
- API de puertos para plugins (WASM Component Model); sandbox y permisos
- Empaquetado desktop opcional (Tauri) y features Plus/Enterprise

Acuerdos técnicos
- Hexagonal: core sin web-sys/wasm-bindgen; adaptadores sólo fuera de core
- Bucle: requestAnimationFrame para render + fixed timestep para lógica
- Serialización: JSON con schema_version y migraciones expuestas por core
- WASM gating: cfg_if y no-ops en targets no wasm32
