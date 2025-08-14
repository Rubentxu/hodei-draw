
### **Documento de Requisitos de Producto (PRD): Plataforma "Hodei Momentum"**

*Una Pizarra Interactiva de Nueva Generación para Diagramación, Animación y Simulación*

**Versión:** 1.0
**Fecha:** 14 de agosto de 2025

#### **1. Visión y Resumen Ejecutivo**

**1.1. Visión del Producto**
Crear una herramienta de diagramación y pizarra virtual que redefine la interactividad. "Hodei Momentum" irá más allá de los diagramas estáticos para convertirse en un lienzo dinámico donde las ideas, los procesos y los sistemas se pueden visualizar, animar y simular. Su objetivo es ser la herramienta preferida para ingenieros, educadores y creativos que buscan comunicar conceptos complejos de una manera clara y atractiva.

**1.2. Resumen Ejecutivo**
"Hodei Momentum" es una aplicación web de alto rendimiento, construida sobre Rust y WebAssembly (WASM), que combina una interfaz de dibujo intuitiva inspirada en Excalidraw con dos capacidades revolucionarias: un **sistema de animación por componentes** y un **motor de simulación de física 2D integrado**.

Esto permite a los usuarios no solo dibujar diagramas de arquitectura, sino también animar el flujo de datos a través de ellos; no solo crear formas, sino también asignarles propiedades físicas para observar sus interacciones en un entorno simulado. El proyecto se desarrollará bajo un modelo **Open Core**, fomentando una comunidad activa mientras se ofrecen funcionalidades avanzadas para equipos profesionales.

**1.3. Audiencia Objetivo**
* **Ingenieros y Desarrolladores:** Para documentar arquitecturas de software, flujos de CI/CD, infraestructura cloud y algoritmos, utilizando animaciones para explicar procesos dinámicos.
* **Educadores y Estudiantes (STEM):** Para crear simulaciones interactivas de conceptos de física (mecánica, cinemática) y matemáticas, mejorando drásticamente la experiencia de aprendizaje.
* **Diseñadores de UX/UI y Creativos:** Para prototipar flujos de usuario, animar transiciones de interfaz y crear presentaciones visuales dinámicas.

#### **2. Contexto y Oportunidad de Mercado**

El mercado de las pizarras virtuales está dominado por herramientas como Miro y FigJam (enfocadas en la colaboración empresarial) y Excalidraw (enfocada en la simplicidad y la rapidez). Si bien son excelentes, dejan un hueco en el mercado para una herramienta que:

1.  **Priorice el Rendimiento Extremo:** Al utilizar Rust/WASM y un renderizado por GPU con `wgpu`, "Hodei Momentum" ofrecerá una experiencia más fluida y con menor consumo de recursos, incluso en lienzos muy complejos.
2.  **Ofrezca Interactividad Profunda:** Ninguna herramienta líder combina de forma nativa la diagramación con la animación de propiedades y la simulación física. Esta es nuestra principal ventaja competitiva.
3.  **Sea Altamente Extensible:** Diseñada desde cero para ser una plataforma, con un futuro sistema de plugins que permitirá a la comunidad contribuir con nuevas funcionalidades de forma segura.

#### **3. Casos de Uso Clave**

* **Ingeniero DevOps:** Modela una infraestructura de Kubernetes. Usa la biblioteca de iconos de K8s, conecta los servicios y anima el flujo de una petición de usuario desde el Ingress hasta el Pod para una presentación al equipo.
* **Profesor de Física:** Arrastra un objeto "caja" y un objeto "plano inclinado" al lienzo. Asigna masa y fricción a la caja, ajusta el ángulo del plano y la gravedad global. Presiona "Play" para simular y demostrar visualmente las leyes de Newton.
* **Diseñador de Producto:** Dibuja dos pantallas de una aplicación móvil. Utiliza el sistema de animación para crear una transición entre ellas, ajustando las curvas de interpolación para lograr un efecto de "easing" suave, y lo exporta como GIF para adjuntarlo a una historia de usuario.

#### **4. Arquitectura Técnica y Pila Tecnológica**

La arquitectura está diseñada para ser modular, de alto rendimiento y basada en el ecosistema de Rust.

| Componente | Tecnología Seleccionada | Justificación |
| :--- | :--- | :--- |
| **Framework de UI** | **Leptos** | Su modelo de reactividad fina basado en señales es ideal para actualizaciones de estado frecuentes y granulares, superando en rendimiento a los enfoques VDOM. |
| **Gestión del Estado del Lienzo** | **`bevy_ecs`** (standalone) | El patrón Entity Component System (ECS) gestionará todos los objetos del lienzo. Es una solución probada, increíblemente rápida y que organiza el código de forma limpia. |
| **Motor de Renderizado** | **`wgpu`** + **`lyon`** + **`glyphon`** | `wgpu` para renderizado por GPU (WebGPU/WebGL2). `lyon` para teselación de formas vectoriales. `glyphon` (o similar) para el renderizado de texto de alta calidad. |
| **Sistema de Animación** | **`bevy_tweening`** (o similar) | Una librería de interpolación (tweening) que se integra con `bevy_ecs` para animar las propiedades de los componentes (ej. `Transform`, `Style`) a lo largo del tiempo. |
| **Motor de Física** | **`avian`** (Primario) o **`bevy_rapier`** | **`avian`** es la opción preferida por su diseño nativo para `bevy_ecs`, lo que simplifica la arquitectura. `bevy_rapier` es una alternativa más madura y potente. |
| **Colaboración en Red** | **`matchbox`** (WebRTC) + **`ggrs`** | Proporciona una colaboración P2P de latencia ultra baja mediante *rollback netcode*, una técnica superior a la simple sincronización de estado. |
| **Persistencia de Datos** | **`IndexedDB`** (vía **`rexie`**) | Para una experiencia "local-first", guardando los proyectos directamente en el navegador del usuario de forma segura y eficiente. |
| **Extensibilidad Futura** | **WASM Component Model** | El objetivo a largo plazo es permitir plugins seguros y multi-idioma para extender la aplicación. |

#### **5. Requisitos Funcionales Detallados (FR)**

**FR-Core (Edición y Dibujo):**
* Lienzo infinito con zoom/panorámica fluidos.
* Creación y manipulación de formas básicas (rectángulo, elipse, línea, flecha, polígono).
* Herramienta de texto con soporte para fuentes y estilos básicos.
* Herramienta de dibujo a mano alzada con suavizado.
* Capacidad para importar imágenes y SVG al lienzo.
* Exportación de la vista actual o selección a PNG y SVG.
* Guardado/Carga de proyectos en un formato JSON abierto.

**FR-Animación:**
* Panel de UI para añadir y gestionar animaciones en los elementos seleccionados.
* Capacidad de animar componentes: `Transform` (posición, rotación, escala) y `Style` (color de relleno, color de borde, opacidad).
* Soporte para múltiples funciones de interpolación (lineal, ease-in, ease-out, elastic, bounce).
* Una "timeline" conceptual para encadenar animaciones y crear secuencias.
* Controles de reproducción para previsualizar animaciones (Play, Pausa, Reset).

**FR-Física:**
* Un "Modo Simulación" que se puede activar/desactivar en el lienzo.
* Un componente `RigidBody` para asignar propiedades físicas: masa, fricción, restitución (rebote), gravedad individual.
* Un componente `Collider` que define la forma física para la detección de colisiones.
* Panel de UI para configurar las propiedades físicas y los ajustes globales de la simulación (ej. gravedad global).
* Capacidad de añadir "Juntas" (joints) para conectar cuerpos (ej. juntas fijas, de resorte).

#### **6. Requisitos No Funcionales (NFR)**

* **Rendimiento:** Latencia de renderizado inferior a 16ms (60 FPS) en todo momento. Tiempo de carga inicial optimizado mediante `wasm-opt` y compresión.
* **Compatibilidad:** Funcionalidad completa en las últimas versiones de Chrome, Firefox y Safari. Compilaciones de escritorio para Windows, macOS y Linux.
* **Seguridad:** Colaboración en tiempo real con cifrado de extremo a extremo (proporcionado por WebRTC). Arquitectura de plugins futura completamente sandboxed.
* **Usabilidad:** La interfaz debe ser intuitiva y descubrible, manteniendo la simplicidad para las tareas básicas.

#### **7. Plan de Desarrollo por Fases (Roadmap)**

**Fase 1: El Mejor Excalidraw (MVP)**
* **Objetivo:** Crear una base sólida y una experiencia de dibujo superior.
* **Funcionalidades:**
    * Toda la funcionalidad del módulo **FR-Core**.
    * Arquitectura base con Leptos, `bevy_ecs` y `wgpu` completamente funcional.
    * Persistencia local en `IndexedDB`.
    * Despliegue inicial de la aplicación web.

**Fase 2: El Lienzo Interactivo**
* **Objetivo:** Introducir las funcionalidades que definen el producto.
* **Funcionalidades:**
    * Implementación completa del sistema de **animación (FR-Animación)**.
    * Implementación del sistema de **física (FR-Física)** con `avian`.
    * Implementación de la **colaboración en tiempo real** básica con `ggrs`.

**Fase 3: La Plataforma Abierta**
* **Objetivo:** Expandir la herramienta para convertirla en un ecosistema.
* **Funcionalidades:**
    * Sistema de bibliotecas de componentes (Cloud, UML, etc.).
    * Inicio del desarrollo de la **arquitectura de plugins (WASM Component Model)**.
    * Lanzamiento de las primeras funcionalidades **Plus/Enterprise** (espacios de trabajo en la nube, gestión de equipos) para el modelo Open Core.