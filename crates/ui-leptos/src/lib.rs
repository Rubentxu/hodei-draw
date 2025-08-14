//! UI basada en Leptos (CSR) para Hodei Momentum

#[cfg(target_arch = "wasm32")]
use leptos::*;
#[cfg(target_arch = "wasm32")]
use leptos::prelude::create_effect;
#[cfg(target_arch = "wasm32")]
use leptos::mount::mount_to_body;
#[cfg(target_arch = "wasm32")]
use leptos::prelude::{
    ElementChild,
    StyleAttribute,
    signal,
    OnAttribute,
    GlobalAttributes,
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
use web_sys::{console, HtmlCanvasElement};

#[cfg(target_arch = "wasm32")]
fn resize_canvas(canvas: &HtmlCanvasElement) {
    let win = window();
    let dpr = win.device_pixel_ratio();
    let Ok(inner_w) = win.inner_width() else { return };
    let Ok(inner_h) = win.inner_height() else { return };
    let Some(wf) = inner_w.as_f64() else { return };
    let Some(hf) = inner_h.as_f64() else { return };
    let w = (wf * dpr).round() as u32;
    let h = (hf * dpr).round() as u32;
    canvas.set_width(w);
    canvas.set_height(h);
    // estilo ocupa todo el contenedor (evitar conflicto con trait StyleAttribute)
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

    // Handlers de puntero básicos
    let drag_start_down = drag_start.clone();
    let on_pointer_down = move |ev: leptos::ev::PointerEvent| {
        let x = ev.offset_x() as f32; // CSS px
        let y = ev.offset_y() as f32; // CSS px
        console::log_1(&format!("pointerdown(canvas): css=({}, {})", x, y).into());
        drag_start_down.set(Some((x, y)));
    };

    let on_pointer_move = move |ev: leptos::ev::PointerEvent| {
        if ev.buttons() & 1 == 1 {
            // Lógica de previsualización eliminada para evitar conflicto de renderers.
            // El renderer WebGPU es la única fuente de verdad.
        }
    };

    let drag_start_up = drag_start.clone();
    let on_pointer_up = move |ev: leptos::ev::PointerEvent| {
        let ex = ev.offset_x() as f32; // CSS px
        let ey = ev.offset_y() as f32; // CSS px
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
            } // Si no supera umbral, tratamos como click: no se crea rectángulo
        }
        drag_start_up.set(None);
    };

    // Ajustar tamaño del canvas en mount y en resize del viewport
    create_effect(move |_| {
        use leptos::prelude::document;
        let doc = document();
        if let Some(elem) = doc.get_element_by_id("main-canvas") {
            if let Ok(canvas) = elem.dyn_into::<HtmlCanvasElement>() {
                resize_canvas(&canvas);
                // Suscribir a resize una vez
                let win = window();
                let cb = Closure::wrap(Box::new({
                    let canvas = canvas.clone();
                    move || {
                        resize_canvas(&canvas);
                    }
                }) as Box<dyn FnMut()>);
                let _ = win.add_event_listener_with_callback("resize", cb.as_ref().unchecked_ref());
                // Mantener el callback vivo mientras viva el componente
                cb.forget();
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
                <button on:click=move |_| {
                    let win = window();
                    match Reflect::get(&win, &JsValue::from_str("force_webgpu")) {
                        Ok(v) => if let Ok(func) = v.dyn_into::<Function>() { let _ = func.call0(&JsValue::NULL); },
                        Err(_) => console::warn_1(&"force_webgpu no disponible".into()),
                    }
                }>"WebGPU"</button>
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
