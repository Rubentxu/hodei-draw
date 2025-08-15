//! UI basada en Leptos (CSR) para Hodei Momentum

#[cfg(target_arch = "wasm32")]
use leptos::*;
#[cfg(target_arch = "wasm32")]
use momentum_design_system::{
    Theme, 
    theme_provider::ThemeProvider,
    icons::{Icon, IconType, IconSize},
    toolbar::{FloatingToolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator, Sidebar, SidebarButton, ThemeToggle}
};
#[cfg(target_arch = "wasm32")]
// create_effect está deprecado; usar Effect::new
use leptos::prelude::Effect;
#[cfg(target_arch = "wasm32")]
use leptos::mount::mount_to_body;
#[cfg(target_arch = "wasm32")]
use leptos::prelude::{
    ElementChild,
    signal,
    OnAttribute,
    GlobalAttributes,
    PropAttribute,
    ClassAttribute,
    StyleAttribute,
    Get,
    Set,
    GetUntracked,
    window,
    IntoView,
};
#[cfg(target_arch = "wasm32")]
use leptos::wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use leptos::wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use leptos::wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};
#[cfg(target_arch = "wasm32")]
use web_sys::{console, HtmlCanvasElement, Element, Event, PointerEvent};

#[cfg(target_arch = "wasm32")]
fn event_to_canvas_css(ev: &leptos::ev::PointerEvent) -> Option<(f32, f32)> {
    use leptos::prelude::document;
    let doc = document();
    let elem = doc.get_element_by_id("main-canvas").and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())?;
    let rect = elem.get_bounding_client_rect();
    // Restar las coordenadas del canvas para obtener coordenadas relativas al canvas
    let x = ev.client_x() as f64 - rect.left();
    let y = ev.client_y() as f64 - rect.top();
    Some((x as f32, y as f32))
}

#[cfg(target_arch = "wasm32")]
fn resize_canvas(canvas: &HtmlCanvasElement) {
    let win = window();
    let dpr = win.device_pixel_ratio();
    // Usa el tamaño real del canvas en el layout (CSS px)
    let rect = canvas.get_bounding_client_rect();
    let css_w = rect.width().max(0.0);
    let css_h = rect.height().max(0.0);
    let w = (css_w * dpr).round() as u32;
    let h = (css_h * dpr).round() as u32;
    if canvas.width() != w { canvas.set_width(w); }
    if canvas.height() != h { canvas.set_height(h); }
    // Asegurar que el canvas ocupa su contenedor
    let _ = canvas.set_attribute("style", "width:100%; height:100%; display:block;");
}

#[cfg(target_arch = "wasm32")]
#[component]
pub fn App() -> impl IntoView {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Tool { Select, Rect, Ellipse, Line }

    #[derive(Clone, Debug)]
    enum PreviewShape {
        Rect { x: f32, y: f32, w: f32, h: f32 },
        Ellipse { cx: f32, cy: f32, rx: f32, ry: f32 },
        Line { x1: f32, y1: f32, x2: f32, y2: f32 },
    }

    // Umbral mínimo de arrastre para considerar creación (evitar click simple -> rect diminuto)
    const DRAG_THRESHOLD: f32 = 4.0;
    let (tool, set_tool) = signal(Tool::Select);
    // Estado local para drag-to-create (coords en CSS px relativos al canvas)
    let drag_start = std::rc::Rc::new(std::cell::Cell::new(None::<(f32, f32)>));
    // Previsualización de forma durante arrastre
    let (drag_preview, set_drag_preview) = signal::<Option<PreviewShape>>(None);
    // Indicadores de soporte/estado
    let (has_webgpu, set_has_webgpu) = signal(false);
    let (renderer_name, set_renderer_name) = signal(String::from("Canvas2D"));
    let (dpr, set_dpr) = signal(window().device_pixel_ratio());
    
    // Theme provider setup
    let theme_provider = ThemeProvider::new();
    let (current_theme, set_current_theme) = signal(theme_provider.effective_theme());

    // Estado de movimiento para herramienta Select
    let (is_dragging_selection, set_is_dragging_selection) = signal(false);
    
    // Estado de escalado para herramienta Select
    let (is_scaling, set_is_scaling) = signal(false);
    let (scale_handle_type, set_scale_handle_type) = signal::<Option<u8>>(None);
    
    // Estado del cursor para feedback visual
    let (cursor_state, set_cursor_state) = signal("default".to_string());
    
    // Handlers de puntero básicos
    let drag_start_down = drag_start.clone();
    let on_pointer_down = move |ev: leptos::ev::PointerEvent| {
        let (x, y) = event_to_canvas_css(&ev).unwrap_or((ev.offset_x() as f32, ev.offset_y() as f32));
        console::log_1(&format!("pointerdown(canvas): css=({}, {})", x, y).into());
        // Activar Pointer Capture para no perder pointerup si salimos del canvas
        if let Some(target) = ev.current_target() {
            if let Ok(elem) = target.dyn_into::<Element>() {
                let _ = elem.set_pointer_capture(ev.pointer_id());
            }
        }
        
        // Si es herramienta Select, manejar selección y posible inicio de movimiento
        if tool.get_untracked() == Tool::Select {
            // Limpiar estado previo
            set_is_scaling.set(false);
            set_scale_handle_type.set(None);
            set_is_dragging_selection.set(false);
            set_cursor_state.set("default".to_string());
            
            let ctrl_key = ev.ctrl_key();
            let shift_key = ev.shift_key();
            let win = window();
            let global: JsValue = win.into();
            if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_pointer_down_with_modifiers")) {
                if let Ok(func) = func_val.dyn_into::<Function>() {
                    let args = js_sys::Array::new();
                    args.push(&JsValue::from_f64(x as f64));
                    args.push(&JsValue::from_f64(y as f64));
                    args.push(&JsValue::from_bool(ctrl_key));
                    args.push(&JsValue::from_bool(shift_key));
                    
                    // Procesar respuesta inmediata
                    if let Ok(result_val) = func.apply(&JsValue::NULL, &args) {
                        // Extraer clicked_handle_type de la respuesta
                        if let Ok(clicked_handle) = Reflect::get(&result_val, &JsValue::from_str("clicked_handle_type")) {
                            if !clicked_handle.is_undefined() && !clicked_handle.is_null() {
                                if let Some(handle_type) = clicked_handle.as_f64() {
                                    let handle_type_u8 = handle_type as u8;
                                    set_scale_handle_type.set(Some(handle_type_u8));
                                    console::log_1(&format!("UI: Handle clicked immediately, type: {}", handle_type_u8).into());
                                    
                                    // Setear cursor apropiado basado en handle type
                                    let cursor = match handle_type_u8 {
                                        0 => "nw-resize", // TopLeft
                                        1 => "ne-resize", // TopRight  
                                        2 => "sw-resize", // BottomLeft
                                        3 => "se-resize", // BottomRight
                                        4 => "n-resize",  // Top
                                        5 => "e-resize",  // Right
                                        6 => "s-resize",  // Bottom
                                        7 => "w-resize",  // Left
                                        _ => "default",
                                    };
                                    set_cursor_state.set(cursor.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        drag_start_down.set(Some((x, y)));
        set_drag_preview.set(None);
    };

    // Clonado para usar dentro de on_pointer_move
    let drag_start_move = drag_start.clone();

    let on_pointer_move = move |ev: leptos::ev::PointerEvent| {
        let (ex, ey) = event_to_canvas_css(&ev).unwrap_or((ev.offset_x() as f32, ev.offset_y() as f32));
        
        // HOVER DETECTION (cuando no se está arrastrando)
        if ev.buttons() == 0 && tool.get_untracked() == Tool::Select {
            // Detectar hover sobre handles (prioridad)
            let win = window();
            let global: JsValue = win.into();
            let mut handle_detected = false;
            
            // Llamar a detect_handle_click para ver si hay un handle bajo el cursor
            if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_detect_handle_hover")) {
                if let Ok(func) = func_val.dyn_into::<Function>() {
                    let args = js_sys::Array::new();
                    args.push(&JsValue::from_f64(ex as f64));
                    args.push(&JsValue::from_f64(ey as f64));
                    
                    if let Ok(result_val) = func.apply(&JsValue::NULL, &args) {
                        if !result_val.is_undefined() && !result_val.is_null() {
                            if let Some(handle_type) = result_val.as_f64() {
                                let handle_type_u8 = handle_type as u8;
                                let cursor = match handle_type_u8 {
                                    0 => "nw-resize", // TopLeft
                                    1 => "ne-resize", // TopRight  
                                    2 => "sw-resize", // BottomLeft
                                    3 => "se-resize", // BottomRight
                                    4 => "n-resize",  // Top
                                    5 => "e-resize",  // Right
                                    6 => "s-resize",  // Bottom
                                    7 => "w-resize",  // Left
                                    _ => "default",
                                };
                                set_cursor_state.set(cursor.to_string());
                                handle_detected = true;
                            }
                        }
                    }
                }
            }
            
            // Si no hay handle, detectar hover sobre shapes
            if !handle_detected {
                if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_detect_shape_hover")) {
                    if let Ok(func) = func_val.dyn_into::<Function>() {
                        let args = js_sys::Array::new();
                        args.push(&JsValue::from_f64(ex as f64));
                        args.push(&JsValue::from_f64(ey as f64));
                        
                        if let Ok(result_val) = func.apply(&JsValue::NULL, &args) {
                            if let Some(has_shape) = result_val.as_bool() {
                                if has_shape {
                                    set_cursor_state.set("grab".to_string());
                                } else {
                                    set_cursor_state.set("default".to_string());
                                }
                            }
                        }
                    }
                } else {
                    set_cursor_state.set("default".to_string());
                }
            }
        }
        
        // DRAG LOGIC (cuando se está arrastrando)
        if ev.buttons() & 1 == 1 {
            if let Some((sx, sy)) = drag_start_move.get() {
                let (ex, ey) = event_to_canvas_css(&ev).unwrap_or((ev.offset_x() as f32, ev.offset_y() as f32));
                let dx_abs = (ex - sx).abs();
                let dy_abs = (ey - sy).abs();
                
                if dx_abs > DRAG_THRESHOLD || dy_abs > DRAG_THRESHOLD {
                    match tool.get_untracked() {
                        Tool::Select => {
                            // Verificar si estamos escalando (decidido inmediatamente en pointer_down)
                            if let Some(handle_type) = scale_handle_type.get_untracked() {
                                // MODO ESCALADO
                                if !is_scaling.get_untracked() {
                                    set_is_scaling.set(true);
                                    let win = window();
                                    let global: JsValue = win.into();
                                    if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_scale_start")) {
                                        if let Ok(func) = func_val.dyn_into::<Function>() {
                                            let args = js_sys::Array::new();
                                            args.push(&JsValue::from_f64(handle_type as f64));
                                            args.push(&JsValue::from_f64(sx as f64));
                                            args.push(&JsValue::from_f64(sy as f64));
                                            let _ = func.apply(&JsValue::NULL, &args);
                                        }
                                    }
                                    console::log_1(&"UI: Started scaling mode".into());
                                }
                                
                                // Enviar update de escalado con delta relativo al punto de inicio
                                let dx = ex - sx;
                                let dy = ey - sy;
                                let win = window();
                                let global: JsValue = win.into();
                                if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_scale_update")) {
                                    if let Ok(func) = func_val.dyn_into::<Function>() {
                                        let args = js_sys::Array::new();
                                        args.push(&JsValue::from_f64(dx as f64));
                                        args.push(&JsValue::from_f64(dy as f64));
                                        let _ = func.apply(&JsValue::NULL, &args);
                                    }
                                }
                            } else {
                                // MODO MOVIMIENTO - solo si no hay handle activo
                                if !is_dragging_selection.get_untracked() {
                                    set_is_dragging_selection.set(true);
                                    set_cursor_state.set("grabbing".to_string());
                                    let win = window();
                                    let global: JsValue = win.into();
                                    if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_move_start")) {
                                        if let Ok(func) = func_val.dyn_into::<Function>() {
                                            let args = js_sys::Array::new();
                                            args.push(&JsValue::from_f64(sx as f64));
                                            args.push(&JsValue::from_f64(sy as f64));
                                            let _ = func.apply(&JsValue::NULL, &args);
                                        }
                                    }
                                    console::log_1(&"UI: Started dragging mode".into());
                                }
                                
                                // Enviar update de movimiento con delta relativo al punto de inicio
                                let dx = ex - sx;
                                let dy = ey - sy;
                                let win = window();
                                let global: JsValue = win.into();
                                if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_move_update")) {
                                    if let Ok(func) = func_val.dyn_into::<Function>() {
                                        let args = js_sys::Array::new();
                                        args.push(&JsValue::from_f64(dx as f64));
                                        args.push(&JsValue::from_f64(dy as f64));
                                        let _ = func.apply(&JsValue::NULL, &args);
                                    }
                                }
                            }
                            set_drag_preview.set(None);
                        }
                        Tool::Rect => {
                            let x = sx.min(ex);
                            let y = sy.min(ey);
                            let w = dx_abs.max(1.0);
                            let h = dy_abs.max(1.0);
                            set_drag_preview.set(Some(PreviewShape::Rect { x, y, w, h }));
                        }
                        Tool::Ellipse => {
                            let x = sx.min(ex);
                            let y = sy.min(ey);
                            let w = dx_abs.max(1.0);
                            let h = dy_abs.max(1.0);
                            let rx = w / 2.0;
                            let ry = h / 2.0;
                            let cx = x + rx;
                            let cy = y + ry;
                            set_drag_preview.set(Some(PreviewShape::Ellipse { cx, cy, rx, ry }));
                        }
                        Tool::Line => {
                            set_drag_preview.set(Some(PreviewShape::Line { x1: sx, y1: sy, x2: ex, y2: ey }));
                        }
                    }
                } else {
                    set_drag_preview.set(None);
                }
            }
        }
    };

    let drag_start_up = drag_start.clone();
    let on_pointer_up = move |ev: leptos::ev::PointerEvent| {
        let (ex, ey) = event_to_canvas_css(&ev).unwrap_or((ev.offset_x() as f32, ev.offset_y() as f32));
        console::log_1(&format!("pointerup(canvas): css=({}, {})", ex, ey).into());
        if let Some((sx, sy)) = drag_start_up.get() {
            let dx = (ex - sx).abs();
            let dy = (ey - sy).abs();
            if dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD {
                let x = sx.min(ex);
                let y = sy.min(ey);
                let w = dx.max(1.0);
                let h = dy.max(1.0);
                // Llamar a función de creación según la herramienta activa
                match tool.get_untracked() {
                    Tool::Rect => {
                        let win = window();
                        let global: JsValue = win.into();
                        if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_create_rect")) {
                            if let Ok(func) = func_val.dyn_into::<Function>() {
                                console::log_1(&format!("UI: calling ecs_create_rect({}, {}, {}, {})", x, y, w, h).into());
                                let args = js_sys::Array::new();
                                args.push(&JsValue::from_f64(x as f64));
                                args.push(&JsValue::from_f64(y as f64));
                                args.push(&JsValue::from_f64(w as f64));
                                args.push(&JsValue::from_f64(h as f64));
                                let _ = func.apply(&JsValue::NULL, &args);
                            }
                        }
                    }
                    Tool::Ellipse => {
                        let rx = w / 2.0;
                        let ry = h / 2.0;
                        let cx = x + rx; // Centro de la elipse
                        let cy = y + ry;
                        let win = window();
                        let global: JsValue = win.into();
                        if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_create_ellipse")) {
                            if let Ok(func) = func_val.dyn_into::<Function>() {
                                console::log_1(&format!("UI: calling ecs_create_ellipse({}, {}, {}, {})", cx, cy, rx, ry).into());
                                let args = js_sys::Array::new();
                                args.push(&JsValue::from_f64(cx as f64));
                                args.push(&JsValue::from_f64(cy as f64));
                                args.push(&JsValue::from_f64(rx as f64));
                                args.push(&JsValue::from_f64(ry as f64));
                                let _ = func.apply(&JsValue::NULL, &args);
                            }
                        }
                    }
                    Tool::Line => {
                        let win = window();
                        let global: JsValue = win.into();
                        if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_create_line")) {
                            if let Ok(func) = func_val.dyn_into::<Function>() {
                                // Usar coordenadas exactas del drag (sx,sy) -> (ex,ey) para consistencia con preview
                                console::log_1(&format!("UI: calling ecs_create_line({}, {}, {}, {})", sx, sy, ex, ey).into());
                                let args = js_sys::Array::new();
                                args.push(&JsValue::from_f64(sx as f64));
                                args.push(&JsValue::from_f64(sy as f64));
                                args.push(&JsValue::from_f64(ex as f64));
                                args.push(&JsValue::from_f64(ey as f64));
                                let _ = func.apply(&JsValue::NULL, &args);
                            }
                        }
                    }
                    Tool::Select => {
                        // Finalizar escalado si estaba activo
                        if is_scaling.get_untracked() {
                            let win = window();
                            let global: JsValue = win.into();
                            if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_scale_end")) {
                                if let Ok(func) = func_val.dyn_into::<Function>() {
                                    let _ = func.apply(&JsValue::NULL, &js_sys::Array::new());
                                }
                            }
                            set_is_scaling.set(false);
                            set_scale_handle_type.set(None);
                            console::log_1(&"UI: Ended scaling mode".into());
                        }
                        // Finalizar movimiento si estaba activo
                        else if is_dragging_selection.get_untracked() {
                            let win = window();
                            let global: JsValue = win.into();
                            if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_move_end")) {
                                if let Ok(func) = func_val.dyn_into::<Function>() {
                                    let _ = func.apply(&JsValue::NULL, &js_sys::Array::new());
                                }
                            }
                            set_is_dragging_selection.set(false);
                            console::log_1(&"UI: Ended dragging mode".into());
                        }
                        
                        // Resetear cursor a default
                        set_cursor_state.set("default".to_string());
                    }
                }
            } // Si no supera umbral, tratamos como click: no se crea rectángulo
        }
        drag_start_up.set(None);
        set_drag_preview.set(None);
    };

    // Efecto para cambiar cursor dinámicamente (aplicar al canvas directamente)
    Effect::new(move |_| {
        let cursor = cursor_state.get();
        let win = window();
        if let Some(document) = win.document() {
            // Buscar el elemento canvas y aplicar el cursor directamente
            if let Ok(canvas) = document.query_selector("canvas") {
                if let Some(canvas_element) = canvas {
                    if let Ok(html_element) = canvas_element.dyn_into::<web_sys::HtmlElement>() {
                        let style = html_element.style();
                        let _ = style.set_property("cursor", &cursor);
                    }
                }
            }
        }
    });

    // Ajustar tamaño del canvas en mount y en resize del viewport
    Effect::new(move |_| {
        use leptos::prelude::document;
        let doc = document();
        if let Some(elem) = doc.get_element_by_id("main-canvas") {
            if let Ok(canvas) = elem.dyn_into::<HtmlCanvasElement>() {
                resize_canvas(&canvas);
                // Suscribir a resize una vez
                let win = window();
                let cb = Closure::wrap(Box::new({
                    let canvas = canvas.clone();
                    let set_dpr = set_dpr.clone();
                    move || {
                        resize_canvas(&canvas);
                        // Actualizar DPR en la UI
                        set_dpr.set(window().device_pixel_ratio());
                    }
                }) as Box<dyn FnMut()>);
                let _ = win.add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref());
                // Mantener el callback vivo mientras viva el componente
                cb.forget();

                // Detectar soporte WebGPU
                let has = js_sys::Reflect::get(&win.navigator().into(), &JsValue::from_str("gpu"))
                    .ok()
                    .map(|v| !v.is_undefined() && !v.is_null())
                    .unwrap_or(false);
                set_has_webgpu.set(has);

                // Inicializar renderer_name desde window.renderer_name si existe
                if let Ok(v) = js_sys::Reflect::get(&win, &JsValue::from_str("renderer_name")) {
                    if let Some(s) = v.as_string() {
                        set_renderer_name.set(s);
                    }
                }

                // Escuchar cambios de renderer mediante Event("renderer-changed"). Leer window.renderer_name.
                let set_renderer_name_evt = set_renderer_name.clone();
                let evt_cb = Closure::wrap(Box::new(move |_ev: Event| {
                    let win = window();
                    if let Ok(v) = js_sys::Reflect::get(&win, &JsValue::from_str("renderer_name")) {
                        if let Some(s) = v.as_string() {
                            set_renderer_name_evt.set(s);
                        }
                    }
                }) as Box<dyn FnMut(Event)>);
                let _ = doc.add_event_listener_with_callback("renderer-changed", evt_cb.as_ref().unchecked_ref());
                evt_cb.forget();

                // Suscribir a pointerup a nivel de documento por si el mouse se suelta fuera del canvas
                // Creamos un listener que finaliza el drag si está activo
                let drag_start_for_doc = drag_start.clone();
                let set_drag_preview_for_doc = set_drag_preview;
                let tool_for_doc = tool;
                let canvas_for_doc = canvas.clone();
                let up_cb = Closure::wrap(Box::new(move |ev: Event| {
                    // Intentar convertir a PointerEvent
                    if let Ok(pev) = ev.dyn_into::<PointerEvent>() {
                        if let Some((sx, sy)) = drag_start_for_doc.get() {
                            // Calcular coords CSS px a partir del canvas y el PointerEvent nativo
                            let rect = canvas_for_doc.get_bounding_client_rect();
                            let ex = pev.client_x() as f32 - rect.left() as f32;
                            let ey = pev.client_y() as f32 - rect.top() as f32;
                            let dx = (ex - sx).abs();
                            let dy = (ey - sy).abs();
                            if dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD {
                                let x = sx.min(ex);
                                let y = sy.min(ey);
                                let w = dx.max(1.0);
                                let h = dy.max(1.0);
                                match tool_for_doc.get_untracked() {
                                    Tool::Rect => {
                                        let win = window();
                                        let global: JsValue = win.into();
                                        if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_create_rect")) {
                                            if let Ok(func) = func_val.dyn_into::<Function>() {
                                                console::log_1(&format!("DOC: ecs_create_rect({}, {}, {}, {})", x, y, w, h).into());
                                                let args = js_sys::Array::new();
                                                args.push(&JsValue::from_f64(x as f64));
                                                args.push(&JsValue::from_f64(y as f64));
                                                args.push(&JsValue::from_f64(w as f64));
                                                args.push(&JsValue::from_f64(h as f64));
                                                let _ = func.apply(&JsValue::NULL, &args);
                                            }
                                        }
                                    }
                                    Tool::Ellipse => {
                                        let rx = w / 2.0;
                                        let ry = h / 2.0;
                                        let cx = x + rx;
                                        let cy = y + ry;
                                        let win = window();
                                        let global: JsValue = win.into();
                                        if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_create_ellipse")) {
                                            if let Ok(func) = func_val.dyn_into::<Function>() {
                                                console::log_1(&format!("DOC: ecs_create_ellipse({}, {}, {}, {})", cx, cy, rx, ry).into());
                                                let args = js_sys::Array::new();
                                                args.push(&JsValue::from_f64(cx as f64));
                                                args.push(&JsValue::from_f64(cy as f64));
                                                args.push(&JsValue::from_f64(rx as f64));
                                                args.push(&JsValue::from_f64(ry as f64));
                                                let _ = func.apply(&JsValue::NULL, &args);
                                            }
                                        }
                                    }
                                    Tool::Line => {
                                        let win = window();
                                        let global: JsValue = win.into();
                                        if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_create_line")) {
                                            if let Ok(func) = func_val.dyn_into::<Function>() {
                                                // Usar coordenadas exactas del drag (sx,sy) -> (ex,ey) para consistencia con preview
                                                console::log_1(&format!("DOC: ecs_create_line({}, {}, {}, {})", sx, sy, ex, ey).into());
                                                let args = js_sys::Array::new();
                                                args.push(&JsValue::from_f64(sx as f64));
                                                args.push(&JsValue::from_f64(sy as f64));
                                                args.push(&JsValue::from_f64(ex as f64));
                                                args.push(&JsValue::from_f64(ey as f64));
                                                let _ = func.apply(&JsValue::NULL, &args);
                                            }
                                        }
                                    }
                                    Tool::Select => {
                                        // Finalizar movimiento si estaba activo
                                        if is_dragging_selection.get_untracked() {
                                            let win = window();
                                            let global: JsValue = win.into();
                                            if let Ok(func_val) = Reflect::get(&global, &JsValue::from_str("ecs_move_end")) {
                                                if let Ok(func) = func_val.dyn_into::<Function>() {
                                                    let _ = func.apply(&JsValue::NULL, &js_sys::Array::new());
                                                }
                                            }
                                            set_is_dragging_selection.set(false);
                                        }
                                    }
                                }
                            }
                            // Limpiar estado tras cualquier pointerup
                            drag_start_for_doc.set(None);
                            set_drag_preview_for_doc.set(None);
                        }
                    }
                }) as Box<dyn FnMut(Event)>);
                let _ = doc.add_event_listener_with_callback("pointerup", up_cb.as_ref().unchecked_ref());
                up_cb.forget();
            }
        }
    });

    view! {
        <div id="app" class="hodei-app">
            // Sidebar izquierdo estilo Excalidraw
            <Sidebar>
                <SidebarButton 
                    icon=IconType::Menu 
                    tooltip="Menú"
                />
                <SidebarButton 
                    icon=IconType::New 
                    tooltip="Nuevo"
                />
                <SidebarButton 
                    icon=IconType::Open 
                    tooltip="Abrir"
                />
                <SidebarButton 
                    icon=IconType::Save 
                    tooltip="Guardar"
                />
                <SidebarButton 
                    icon=IconType::Export 
                    tooltip="Exportar"
                />
                
                <div class="h-2" />
                
                <SidebarButton 
                    icon=IconType::Settings 
                    tooltip="Configuración"
                />
            </Sidebar>
            
            // Área del canvas
            <div class="canvas-area">
                <canvas
                    id="main-canvas"
                    class="main-canvas"
                    on:pointerdown=on_pointer_down
                    on:pointermove=on_pointer_move
                    on:pointerup=on_pointer_up
                />
                
                // Overlay de previsualización durante el arrastre
                {move || {
                    if let Some(preview) = drag_preview.get() {
                        match preview {
                            PreviewShape::Rect { x, y, w, h } => {
                                view! {
                                    <div 
                                        class="preview-rect"
                                        style=format!("left: {}px; top: {}px; width: {}px; height: {}px;", x, y, w, h)
                                    />
                                }.into_view()
                            }
                            PreviewShape::Ellipse { cx, cy, rx, ry } => {
                                let x = cx - rx;
                                let y = cy - ry;
                                let w = rx * 2.0;
                                let h = ry * 2.0;
                                view! {
                                    <div 
                                        class="preview-ellipse"
                                        style=format!("left: {}px; top: {}px; width: {}px; height: {}px;", x, y, w, h)
                                    />
                                }.into_view()
                            }
                            PreviewShape::Line { x1, y1, x2, y2 } => {
                                let dx = x2 - x1;
                                let dy = y2 - y1;
                                let length = (dx * dx + dy * dy).sqrt();
                                let angle = dy.atan2(dx) * 180.0 / std::f32::consts::PI;
                                view! {
                                    <div 
                                        class="preview-line"
                                        style=format!(
                                            "left: {}px; top: {}px; width: {}px; height: 2px; transform: rotate({}deg); transform-origin: 0 center;", 
                                            x1, y1, length, angle
                                        )
                                    />
                                }.into_view()
                            }
                        }
                    } else {
                        view! { <div class="hidden" style=String::new() /> }.into_view()
                    }
                }}
            </div>
            
            // Floating toolbar principal
            <FloatingToolbar>
                <ToolbarGroup>
                    <ToolbarButton 
                        icon=IconType::Select
                        tooltip="Seleccionar (V)"
                        selected=Box::new(move || tool.get() == Tool::Select)
                        on_click=Box::new(move || set_tool.set(Tool::Select))
                    />
                    <ToolbarButton 
                        icon=IconType::Rectangle
                        tooltip="Rectángulo (R)"
                        selected=Box::new(move || tool.get() == Tool::Rect)
                        on_click=Box::new(move || set_tool.set(Tool::Rect))
                    />
                    <ToolbarButton 
                        icon=IconType::Ellipse
                        tooltip="Elipse (O)"
                        selected=Box::new(move || tool.get() == Tool::Ellipse)
                        on_click=Box::new(move || set_tool.set(Tool::Ellipse))
                    />
                    <ToolbarButton 
                        icon=IconType::Arrow
                        tooltip="Flecha (A)"
                        disabled=true
                    />
                    <ToolbarButton 
                        icon=IconType::Line
                        tooltip="Línea (L)"
                        selected=Box::new(move || tool.get() == Tool::Line)
                        on_click=Box::new(move || set_tool.set(Tool::Line))
                    />
                    <ToolbarButton 
                        icon=IconType::Pen
                        tooltip="Lápiz (P)"
                        disabled=true
                    />
                    <ToolbarButton 
                        icon=IconType::Text
                        tooltip="Texto (T)"
                        disabled=true
                    />
                </ToolbarGroup>
                
                <ToolbarSeparator />
                
                <ToolbarGroup>
                    <ToolbarButton 
                        icon=IconType::Eraser
                        tooltip="Borrador"
                        disabled=true
                    />
                    <ToolbarButton 
                        icon=IconType::Hand
                        tooltip="Mano (H)"
                        disabled=true
                    />
                </ToolbarGroup>
                
                <ToolbarSeparator />
                
                <ToolbarGroup>
                    <ToolbarButton 
                        icon=IconType::Undo
                        tooltip="Deshacer (Ctrl+Z)"
                        disabled=true
                    />
                    <ToolbarButton 
                        icon=IconType::Redo
                        tooltip="Rehacer (Ctrl+Y)"
                        disabled=true
                    />
                </ToolbarGroup>
                
                <ToolbarSeparator />
                
                <ToolbarGroup>
                    <ThemeToggle 
                        current_theme=current_theme
                        on_theme_change=move |new_theme| {
                            theme_provider.set_theme(new_theme);
                            set_current_theme.set(new_theme);
                        }
                    />
                </ToolbarGroup>
            </FloatingToolbar>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
pub fn mount_app() {
    mount_to_body(App);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mount_app() {}