# Hodei Momentum — Roadmap General

Version: 1.0
Fecha: 2025-08-14

Resumen
- Producto: pizarra interactiva con dibujo, animación y simulación 2D.
- Principios: arquitectura hexagonal, clean code, performance 60 FPS, local-first.
- Pila: Rust+WASM, Leptos, bevy_ecs, wgpu+lyon+glyphon, avian/bevy_rapier, matchbox+ggrs, rexie/IndexedDB.

Fases y Objetivos
1) Fase 1 — El Mejor Excalidraw (MVP)
   - Meta: base sólida de edición y render con persistencia local.
   - Entregable: demo web pública con FR-Core completos y 60 FPS.
2) Fase 2 — Lienzo Interactivo
   - Meta: animación por componentes y física 2D integradas; colaboración básica P2P.
   - Entregable: demos de animación y simulación; sesión colaborativa simple.
3) Fase 3 — Plataforma Abierta
   - Meta: bibliotecas de componentes, inicio arquitectura de plugins (WASM CM), empaquetado desktop opcional.
   - Entregable: catálogo de librerías (UML/Cloud), SDK inicial de plugins.

Hitos por Fase
Fase 1 (MVP)
- H1.1 Workspace multi-crate y puertos (traits) del dominio
- H1.2 Render vectorial (formas, texto) por GPU con wgpu+lyon+glyphon
- H1.3 Interacciones de edición: seleccionar, mover, escalar, rotar, lápiz libre
- H1.4 Importación SVG, exportación PNG/SVG
- H1.5 Persistencia local (IndexedDB) y formato JSON abierto
- H1.6 Undo/Redo robusto y tests núcleo
- H1.7 Optimización WASM (wasm-opt) y presupuesto de tamaño

Fase 2 (Interactivo)
- H2.1 Timeline y sistema de tweening (easings) sobre Transform/Style
- H2.2 Controles Play/Pause/Reset y previsualización
- H2.3 Física con avian: RigidBody, Collider, juntas básicas, ajustes globales
- H2.4 Sincronización P2P (matchbox+ggrs) con rollback netcode sobre comandos
- H2.5 Determinismo: timestep fijo y auditoría de fuentes no-deterministas
- H2.6 Demos públicas de animación y simulación (educación + ingeniería)

Fase 3 (Plataforma)
- H3.1 Librerías de componentes (UML, Cloud, ítems reutilizables)
- H3.2 API estable de puertos para plugins (WASM Component Model)
- H3.3 Sandbox y permisos; empaquetado desktop (opcional: Tauri)
- H3.4 Features Plus/Enterprise iniciales (workspaces cloud, equipos)

Cronograma sugerido (primeros 3-4 meses)
- Semana 1-2: H1.1, base workspace, puertos del dominio, UI mínima Leptos
- Semana 3-4: H1.2 render básico, texto, pipeline de build (Trunk), RAF loop, fallback Canvas2D cuando WebGPU no está disponible
- Semana 5-6: H1.3 interacciones y sistemas ECS para edición; selección/handles
- Semana 7: H1.4 import/export; snapshots de serialización
- Semana 8: H1.5 IndexedDB y migraciones de esquema
- Semana 9: H1.6 undo/redo, property-based tests
- Semana 10: H1.7 optimización, presupuesto WASM, demo pública MVP
- Semana 11-12: H2.1/H2.2 timeline/easing; controles de reproducción
- Semana 13-14: H2.3 física (integración mínima), demo de colisiones y gravedad
- Semana 15: H2.4/H2.5 colaboración básica + determinismo
- Semana 16: H2.6 demos públicas, hardening y feedback

KPIs y Criterios de Aceptación
- Rendimiento: 60 FPS con 1k entidades simples; latencia <16ms.
- Tamaño WASM: objetivo <3–5 MB gzip en MVP.
- Calidad: >80% cobertura en core crítico; tests de snapshot estables.
- UX: tareas básicas descubiertas en <2 min por usuarios nuevos (test informal).
- Estabilidad: 0 crashes en smoke tests de 30 min.

Riesgos y Mitigaciones
- Falta de WebGPU: fallback inmediato a Canvas2D (implementado) y, a futuro, WebGL2 vía wgpu; degradación controlada. Se incluyeron controles en la UI para forzar Canvas2D/WebGPU y telemetría mínima en consola.
- Texto en WASM: validar glyphon/swash temprano; cache de fuentes.
- Física/Determinismo: fixed timestep, auditoría de FP; fallback a rapier si avian bloquea.
- P2P: escalado limitado; para equipos grandes, plan futuro servidor relay/estado.
- Tamaño WASM: LTO, opt-level=z, feature flags, evitar dependencias pesadas.

Dependencias externas claves
- wgpu, lyon, glyphon/swash, bevy_ecs, avian/bevy_rapier, matchbox, ggrs, rexie, leptos, wasm-bindgen, trunk.

Notas de versión y releases
- Release semántico por fase; changelog en Keep a Changelog; etiquetas git hodei-vX.Y.Z.
