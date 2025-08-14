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
- ✅ **MVP FUNCIONAL COMPLETADO**: Aplicación web totalmente funcional con todas las herramientas básicas
- ✅ **Renderer Canvas2D Completo**: Implementación completa con soporte para Rect, Ellipse, Line, Polygon + estilos avanzados (fill, stroke, dash patterns) + paths vectoriales + texto básico + transformaciones de cámara
- ✅ **Sistema de Selección**: Hit testing preciso para todas las formas con feedback visual (borde azul)
- ✅ **Herramientas UI**: Seleccionar, Rectángulo, Elipse, Línea con drag-to-create funcional
- ✅ **Arquitectura Hexagonal**: Core/ECS/UI/App con puertos bien definidos y separación clara
- ✅ **Fallback Canvas2D**: Automático desde WebGPU con detección de soporte y controles UI
- ✅ **Bug crítico resuelto**: Transform Default corregido (escala 1.0) - renderizado ahora funciona correctamente

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
✅ 1) Infra/workspace y puertos
- ✅ Crates: core, ecs, ui-leptos, apps/app-web
- ✅ Puertos en core: RenderPort completo, StoragePort (interface), ClipboardPort (pendiente)
- ✅ Adaptadores web en app-web (driven) y UI Leptos (driving)

✅ 2) Modelo de dominio (core)
- ✅ Tipos: Document, Shape (Rect/Ellipse/Line/Polygon), Style, Transform, Color
- ✅ IDs fuertes: EntityId
- ✅ Serialización JSON (serde); thiserror para errores

✅ 3) Orquestación ECS (ecs)
- ✅ Recursos: Document, Selection, InputQueue, CanvasSize, CanvasDpr
- ✅ Eventos: PointerDown, CreateRect/Ellipse/Line
- ✅ Sistemas: creación de formas, selección con hit testing, render con selección

✅ 4) Render vectorial (app-web + adapter Renderer)
- ✅ Canvas2D completo con todas las formas, estilos, texto y transformaciones
- ✅ Fallback automático desde WebGPU cuando no disponible
- ✅ Cámara con pan/zoom, DPR support

✅ 5) UI Leptos (ui-leptos)
- ✅ Shell: toolbar con herramientas, canvas host, indicador estado
- ✅ Herramientas: Seleccionar, Rectángulo, Elipse, Línea
- ✅ Drag-to-create funcional con preview
- ✅ Controles Canvas2D/WebGPU con indicador renderer + DPR

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
- ✅ S1-2: workspace, puertos core, UI mínima, loop RAF
- ✅ S3-4: render Canvas2D completo, texto básico, pan/zoom, fallback automático, controles UI e indicador renderer/DPR
- 🔄 S5-6: ✅ selección/crear formas múltiples, 🚧 mover, ⏳ escala/rotar, ⏳ pencil
- ⏳ S7: import/export SVG/PNG
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
