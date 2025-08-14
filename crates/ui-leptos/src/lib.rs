//! UI basada en Leptos (CSR) para Hodei Momentum

#[cfg(target_arch = "wasm32")]
use leptos::*;
#[cfg(target_arch = "wasm32")]
// create_effect está deprecado; usar Effect::new
use leptos::prelude::Effect;
#[cfg(target_arch = "wasm32")]
use leptos::mount::mount_to_body;
#[cfg(target_arch = "wasm32")]
use leptos::prelude::{
    ElementChild,
    StyleAttribute,
    signal,
    OnAttribute,
    GlobalAttributes,
    PropAttribute,
    ClassAttribute,
    Get,
    Set,
    GetUntracked,
    window,
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
    enum Tool { Select, Rect }

    // Umbral mínimo de arrastre para considerar creación (evitar click simple -> rect diminuto)
    const DRAG_THRESHOLD: f32 = 4.0;
    let (tool, set_tool) = signal(Tool::Rect);
    // Estado local para drag-to-create (coords en CSS px relativos al canvas)
    let drag_start = std::rc::Rc::new(std::cell::Cell::new(None::<(f32, f32)>));
    // Previsualización de rectángulo durante arrastre (x,y,w,h en CSS px)
    let (drag_rect, set_drag_rect) = signal::<Option<(f32, f32, f32, f32)>>(None);
    // Indicadores de soporte/estado
    let (has_webgpu, set_has_webgpu) = signal(false);
    let (renderer_name, set_renderer_name) = signal(String::from("Canvas2D"));
    let (dpr, set_dpr) = signal(window().device_pixel_ratio());

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
        drag_start_down.set(Some((x, y)));
        set_drag_rect.set(None);
    };

    // Clonado para usar dentro de on_pointer_move
    let drag_start_move = drag_start.clone();

    let on_pointer_move = move |ev: leptos::ev::PointerEvent| {
        if ev.buttons() & 1 == 1 {
            if let Some((sx, sy)) = drag_start_move.get() {
                let (ex, ey) = event_to_canvas_css(&ev).unwrap_or((ev.offset_x() as f32, ev.offset_y() as f32));
                let dx = (ex - sx).abs();
                let dy = (ey - sy).abs();
                if dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD {
                    let x = sx.min(ex);
                    let y = sy.min(ey);
                    let w = dx.max(1.0);
                    let h = dy.max(1.0);
                    set_drag_rect.set(Some((x, y, w, h)));
                } else {
                    set_drag_rect.set(None);
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
                // Llamar a ecs_create_rect(x,y,w,h)
                if tool.get_untracked() == Tool::Rect {
                    {
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
                            } else {
                                console::warn_1(&"UI: ecs_create_rect no es una función".into());
                            }
                        } else {
                            console::warn_1(&"UI: window.ecs_create_rect no encontrado".into());
                        }
                    }
                }
            } // Si no supera umbral, tratamos como click: no se crea rectángulo
        }
        drag_start_up.set(None);
        set_drag_rect.set(None);
    };

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
                let evt_cb = Closure::wrap(Box::new(move |ev: Event| {
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
                let set_drag_rect_for_doc = set_drag_rect;
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
                                if tool_for_doc.get_untracked() == Tool::Rect {
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
                            }
                            // Limpiar estado tras cualquier pointerup
                            drag_start_for_doc.set(None);
                            set_drag_rect_for_doc.set(None);
                        }
                    }
                }) as Box<dyn FnMut(Event)>);
                let _ = doc.add_event_listener_with_callback("pointerup", up_cb.as_ref().unchecked_ref());
                up_cb.forget();
            }
        }
    });

    view! {
        <div id="app" style="font-family: system-ui, sans-serif;">
            <header style="display:flex;gap:.5rem;align-items:center;padding:.5rem 1rem;border-bottom:1px solid #e5e7eb;">
                <strong>Hodei Momentum</strong>
                <span style="flex:1"></span>
                // Controles de renderer
                <button on:click=move |_| {
                    let win = window();
                    match Reflect::get(&win, &JsValue::from_str("force_canvas2d")) {
                        Ok(v) => if let Ok(func) = v.dyn_into::<Function>() { let _ = func.call0(&JsValue::NULL); },
                        Err(_) => console::warn_1(&"force_canvas2d no disponible".into()),
                    }
                }>"Canvas2D"</button>
                <button
                    on:click=move |_| {
                    let win = window();
                    match Reflect::get(&win, &JsValue::from_str("force_webgpu")) {
                        Ok(v) => if let Ok(func) = v.dyn_into::<Function>() { let _ = func.call0(&JsValue::NULL); },
                        Err(_) => console::warn_1(&"force_webgpu no disponible".into()),
                    }
                }
                    prop:disabled=move || !has_webgpu.get()
                    title=move || if has_webgpu.get() { "Cambiar a WebGPU".to_string() } else { "WebGPU no soportado en este navegador".to_string() }
                >"WebGPU"</button>
                <span style="margin-left:.75rem;color:#6b7280;">
                    {move || format!("Renderer: {} | DPR: {:.2}", renderer_name.get(), dpr.get())}
                </span>
                <div style="margin-left:auto;display:flex;gap:.5rem;">
                    <button
                        on:click=move |_| set_tool.set(Tool::Select)
                        style=move || if tool.get() == Tool::Select { "background:#e5e7eb" } else { "" }
                    >Seleccionar</button>
                    <button
                        on:click=move |_| set_tool.set(Tool::Rect)
                        style=move || if tool.get() == Tool::Rect { "background:#e5e7eb" } else { "" }
                    >Rectángulo</button>
                    <button>Elipse</button>
                    <button>Línea</button>
                    <button>Lápiz</button>
                </div>
            </header>
            <main style="display:flex;height:calc(100vh - 48px);">
                <aside style="width:260px;border-right:1px solid #e5e7eb;padding:.5rem;">Propiedades</aside>
                <section style="flex:1;position:relative;background:#fafafa;min-height:640px;">
                    <canvas
                        id="main-canvas"
                        class="absolute top-0 left-0 w-full h-full"
                        on:pointerdown=on_pointer_down
                        on:pointermove=on_pointer_move
                        on:pointerup=on_pointer_up
                    />
                    // Overlay de previsualización durante el arrastre
                    <div style=move || {
                        if let Some((x,y,w,h)) = drag_rect.get() {
                            format!(
                                "position:absolute; left:{x}px; top:{y}px; width:{w}px; height:{h}px; \
                                 pointer-events:none; border:1px dashed #3b82f6; background:rgba(59,130,246,0.06);"
                            )
                        } else { "display:none".into() }
                    }></div>
                </section>
                <aside style="width:260px;border-left:1px solid #e5e7eb;padding:.5rem;">Capas</aside>
            </main>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
pub fn mount_app() {
    mount_to_body(App);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn mount_app() {}
