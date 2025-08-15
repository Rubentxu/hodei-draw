# Contribuir a Hodei Draw

🎉 **¡Gracias por tu interés en contribuir a Hodei Draw!** 🎉

Damos la bienvenida a contribuciones de todos, ya seas un desarrollador experimentado en Rust o recién estés comenzando con WebAssembly.

## 📋 Tabla de Contenidos

- [Código de Conducta](#código-de-conducta)
- [¿Cómo Puedo Contribuir?](#cómo-puedo-contribuir)
- [Configuración de Desarrollo](#configuración-de-desarrollo)
- [Estándares de Código](#estándares-de-código)
- [Guías de Commit](#guías-de-commit)
- [Proceso de Pull Request](#proceso-de-pull-request)
- [Reporte de Issues](#reporte-de-issues)

## 📜 Código de Conducta

Este proyecto se adhiere a un ambiente amigable, inclusivo y respetuoso. Por favor sé amable y considerado en todas las interacciones.

## 🤝 ¿Cómo Puedo Contribuir?

### 🐛 Reportes de Bugs
- Usa el [template de reporte de bug](.github/ISSUE_TEMPLATE/bug_report.md)
- Proporciona pasos claros de reproducción
- Incluye información del navegador y SO
- Agrega capturas o GIFs si es útil

### 💡 Solicitudes de Características
- Usa el [template de solicitud de característica](.github/ISSUE_TEMPLATE/feature_request.md)
- Explica el caso de uso y comportamiento esperado
- Considera si se alinea con la hoja de ruta del proyecto

### 🛠️ Contribuciones de Código
- Escoge un issue etiquetado como `good first issue` para comenzar
- Haz fork del repositorio y crea una rama de característica
- Escribe código limpio y testeado siguiendo nuestros estándares
- Envía un pull request con una descripción clara

## 🚀 Configuración de Desarrollo

### Prerequisitos

```bash
# Instalar Rust (última versión estable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Agregar target wasm32
rustup target add wasm32-unknown-unknown

# Instalar Trunk (herramienta de build WASM)
cargo install trunk

# Instalar Node.js (para procesamiento CSS)
# Descargar desde https://nodejs.org/ (18+ requerido)
```

### Desarrollo Local

```bash
# Clonar tu fork
git clone https://github.com/TU_USUARIO/hodei-draw.git
cd hodei-draw

# Instalar dependencias
cd apps/app-web && npm install && cd ../..

# Iniciar servidor de desarrollo
make serve
# O
cd apps/app-web && trunk serve --open
```

### Ejecutar Pruebas

```bash
# Ejecutar todas las pruebas
cargo test

# Ejecutar pruebas para crate específico
cargo test -p momentum-core

# Ejecutar con cobertura
cargo tarpaulin --out html
```

## 🎯 Estándares de Código

### Estilo de Código Rust

```bash
# Formatear código (requerido antes del commit)
cargo fmt

# Ejecutar clippy (requerido antes del commit)
cargo clippy -- -D warnings

# Verificar todas las características
cargo clippy --all-features -- -D warnings
```

### Organización del Código

- **Lógica de Dominio**: Colocar en `crates/core/`
- **Componentes ECS**: Colocar en `crates/ecs/`
- **Componentes UI**: Colocar en `crates/ui-leptos/`
- **Sistema de Diseño**: Colocar en `crates/design-system/`

## 📝 Guías de Commit

Seguimos la especificación [Conventional Commits](https://conventionalcommits.org/):

```
tipo(ámbito): descripción breve

[cuerpo opcional]

[pie opcional]
```

### Tipos
- `feat`: Nueva característica
- `fix`: Corrección de bug
- `docs`: Cambios en documentación
- `style`: Cambios de estilo de código (formateo, etc.)
- `refactor`: Refactorización de código
- `test`: Agregar o actualizar pruebas
- `chore`: Tareas de mantenimiento

### Ejemplos

```bash
git commit -m "feat(ui): agregar barra de herramientas de selección"
git commit -m "fix(core): resolver condición de carrera en renderizado"
git commit -m "docs(readme): actualizar instrucciones de instalación"
```

## 🔄 Proceso de Pull Request

### Antes de Enviar

1. **Crear Rama de Característica**
   ```bash
   git checkout -b feature/nombre-de-tu-caracteristica
   ```

2. **Escribir Pruebas**
   - Agregar pruebas unitarias para nueva funcionalidad
   - Asegurar que las pruebas existentes pasen
   - Agregar pruebas de integración si es necesario

3. **Verificaciones de Calidad de Código**
   ```bash
   # Todos deben pasar
   cargo fmt --check
   cargo clippy -- -D warnings  
   cargo test
   ```

## 🐛 Reporte de Issues

### Los Reportes de Bug Deben Incluir:

- **Título Claro**: Descripción concisa del issue
- **Pasos para Reproducir**: Lista numerada de acciones
- **Comportamiento Esperado**: Qué debería pasar
- **Comportamiento Real**: Qué pasa realmente
- **Ambiente**: Navegador, SO, info de versión
- **Capturas**: Si es un issue visual
- **Logs de Consola**: Cualquier mensaje de error

---

**¡Gracias por contribuir a Hodei Draw!** 🎨✨