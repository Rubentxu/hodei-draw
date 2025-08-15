# Hodei Draw - End-to-End Tests

Este directorio contiene los tests E2E para la aplicación Hodei Draw usando **Playwright** para Rust.

## Configuración

### Prerequisitos

1. **Playwright**: Instalar los binarios de navegadores
   ```bash
   # Primero, instalar playwright global (opcional)
   npm install -g playwright
   
   # Instalar navegadores para Playwright
   playwright install chromium
   ```

2. **Aplicación corriendo**: Los tests asumen que la aplicación está corriendo en `http://localhost:8082/hodei-draw/`
   ```bash
   # Desde el directorio raíz del proyecto
   cd apps/app-web
   trunk serve --port 8082
   ```

## Estructura de Tests

```
tests/
├── common/
│   └── mod.rs              # Utilidades compartidas y setup
├── e2e/
│   ├── drawing_tools.rs    # Tests de herramientas de dibujo
│   ├── hover_behavior.rs   # Tests de comportamiento hover/cursor
│   ├── scale_handles.rs    # Tests de interacción con handles de escala
│   └── visual_regression.rs # Tests de regresión visual
├── screenshots/            # Screenshots automáticos para comparación
└── Cargo.toml             # Configuración del proyecto de tests
```

## Ejecutar Tests

### Todos los tests E2E
```bash
cd tests
cargo test
```

### Tests específicos
```bash
# Solo tests de drawing tools
cargo test drawing_tools

# Solo tests de hover behavior
cargo test hover_behavior

# Solo tests de scale handles
cargo test scale_handles

# Solo tests de regresión visual
cargo test visual_regression
```

### Tests en modo verbose (para debugging)
```bash
cargo test -- --nocapture
```

## Tipos de Tests

### 1. Drawing Tools (`drawing_tools.rs`)
- ✅ Creación de rectángulos
- ✅ Creación de múltiples formas (rect, circle, line)
- ✅ Selección de formas
- ✅ Verificación de contenido en canvas

### 2. Hover Behavior (`hover_behavior.rs`)
- ✅ Cambios de cursor en hover sobre formas (`grab`)
- ✅ Cambios de cursor en hover sobre handles (`nw-resize`, `se-resize`, etc.)
- ✅ Prioridad de handles sobre formas
- ✅ Funciones WASM de detección de hover

### 3. Scale Handles (`scale_handles.rs`)
- ✅ Aparición de handles al seleccionar formas
- ✅ Interacción de arrastre con handles
- ✅ Secuencia de eventos de escalado (start/update/end)
- ✅ Testing de todos los tipos de handles

### 4. Visual Regression (`visual_regression.rs`)
- ✅ Screenshots baseline de formas básicas
- ✅ Screenshots de handles de selección
- ✅ Consistencia visual con múltiples selecciones
- ✅ Verificación de precisión de píxeles
- ✅ Consistencia con Device Pixel Ratio

## Funciones WASM Utilizadas

Los tests acceden directamente a las funciones WASM expuestas:

```javascript
// Creación de formas
ecs_create_rect(x, y, width, height)
ecs_create_ellipse(x, y, rx, ry)
ecs_create_line(x1, y1, x2, y2)

// Interacción
ecs_pointer_down(x, y)

// Escalado
ecs_scale_start(handle_type, x, y)
ecs_scale_update(dx, dy)
ecs_scale_end()

// Detección de hover (implementadas en nuestra solución)
ecs_detect_shape_hover(x, y)
ecs_detect_handle_hover(x, y)
```

## Screenshots

Los tests generan automáticamente screenshots en `tests/screenshots/` para:

- **Verificación manual**: Ver el estado visual después de cada test
- **Debugging**: Identificar qué está pasando cuando un test falla
- **Regresión visual**: Comparar cambios visuales entre versiones

### Ejemplos de screenshots generados:
- `rectangle_created.png` - Rectángulo básico creado
- `multiple_shapes.png` - Múltiples formas en canvas
- `shape_selected.png` - Forma seleccionada con handles
- `after_scaling.png` - Resultado después de escalar

## Debugging Tests

### Tests failing?

1. **Verificar que la aplicación está corriendo**:
   ```bash
   curl http://localhost:8082/hodei-draw/
   ```

2. **Revisar screenshots**: Los tests generan screenshots automáticamente para debug

3. **Ejecutar con logs detallados**:
   ```bash
   RUST_LOG=debug cargo test -- --nocapture
   ```

4. **Ejecutar un solo test**:
   ```bash
   cargo test test_create_rectangle -- --nocapture
   ```

### Tests en modo headless vs headed

Por defecto, los tests corren en modo **headed** (con ventana visible) para facilitar el debugging. 

Para cambiar a **headless**, editar en `common/mod.rs`:
```rust
let browser = browser_type
    .launch()
    .headless(true)  // Cambiar a true
    .launch()
    .await?;
```

## Beneficios de estos Tests E2E

1. **Validación completa de UX**: Verificamos que el cursor cambia correctamente
2. **Testing de interacción**: Probamos arrastre, hover, selección real
3. **Regresión visual**: Detectamos cambios no intencionados en rendering
4. **Validación cross-browser**: Pueden ejecutarse en Chrome, Firefox, Safari
5. **CI/CD ready**: Perfectos para integración continua

## Próximos Tests a Implementar

- [ ] Tests de multi-selección con Ctrl+click
- [ ] Tests de keyboard shortcuts
- [ ] Tests de diferentes tamaños de ventana/DPR
- [ ] Tests de performance con muchas formas
- [ ] Tests de undo/redo cuando se implemente

---

**Nota**: Estos tests validan completamente el sistema de hover y scale handles que implementamos, incluyendo las funciones `ecs_detect_handle_hover` y `ecs_detect_shape_hover` que añadimos para el cursor feedback.