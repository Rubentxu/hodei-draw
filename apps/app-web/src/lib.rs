#[cfg(target_arch = "wasm32")]
use momentum_ecs::MomentumEcsApp;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{console, window, Event};
#[cfg(target_arch = "wasm32")]
use momentum_ui_leptos::mount_app;
#[cfg(target_arch = "wasm32")]
use momentum_core::model::Shape;
#[cfg(target_arch = "wasm32")]
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use momentum_core::ports::RenderPort;
#[cfg(all(target_arch = "wasm32", feature = "webgpu"))]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(all(target_arch = "wasm32", feature = "webgpu"))]
mod renderer_webgpu;
#[cfg(all(target_arch = "wasm32", feature = "webgpu"))]
use renderer_webgpu::WebGpuRenderer;
#[cfg(target_arch = "wasm32")]
mod renderer_canvas2d;
#[cfg(target_arch = "wasm32")]
use renderer_canvas2d::Canvas2DRenderer;

#[cfg(target_arch = "wasm32")]
thread_local! {
    static ECS: std::cell::RefCell<Option<MomentumEcsApp>> = const { std::cell::RefCell::new(None) };
    static FRAME: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

#[cfg(target_arch = "wasm32")]
fn announce_renderer(name: &str) {
    if let Some(win) = window() {
        let _ = js_sys::Reflect::set(
            &win,
            &JsValue::from_str("renderer_name"),
            &JsValue::from_str(name),
        );
        if let Some(doc) = win.document() {
            // Dispatch simple Event("renderer-changed"). UI leerá window.renderer_name.
            if let Ok(evt) = Event::new("renderer-changed") {
                let _ = doc.dispatch_event(&evt);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn force_canvas2d() {
    console::log_1(&"force_canvas2d() called".into());
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(elem) = doc.get_element_by_id("main-canvas") {
                if let Ok(canvas) = elem.dyn_into::<HtmlCanvasElement>() {
                    match Canvas2DRenderer::new(canvas) {
                        Ok(renderer) => {
                            ECS.with(|ecs| {
                                if let Some(app) = &mut *ecs.borrow_mut() {
                                    let boxed: Box<dyn RenderPort> = Box::new(renderer);
                                    app.set_renderer(boxed);
                                }
                            });
                            announce_renderer("Canvas2D");
                            console::log_1(&"Canvas2D renderer set".into());
                        }
                        Err(e) => console::log_2(&"Canvas2D init error".into(), &JsValue::from_str(&format!("{e:?}"))),
                    }
                }
            }
        }
    }
}

#[cfg(all(target_arch = "wasm32", feature = "webgpu"))]
#[wasm_bindgen(js_namespace = window)]
pub fn force_webgpu() {
    console::log_1(&"force_webgpu() called".into());
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(elem) = doc.get_element_by_id("main-canvas") {
                if let Ok(canvas) = elem.dyn_into::<HtmlCanvasElement>() {
                    // Evitar intentar WebGPU si no está disponible
                    let has_webgpu = js_sys::Reflect::get(
                        &win.navigator().into(),
                        &JsValue::from_str("gpu"),
                    )
                    .ok()
                    .map(|v| !v.is_undefined() && !v.is_null())
                    .unwrap_or(false);

                    if !has_webgpu {
                        console::info_1(&"WebGPU no soportado; usando Canvas2D".into());
                        match Canvas2DRenderer::new(canvas) {
                            Ok(renderer) => {
                                ECS.with(|ecs| {
                                    if let Some(app) = &mut *ecs.borrow_mut() {
                                        let boxed: Box<dyn RenderPort> = Box::new(renderer);
                                        app.set_renderer(boxed);
                                    }
                                });
                                console::log_1(&"Canvas2D renderer set".into());
                            }
                            Err(e) => console::log_2(&"Canvas2D init error".into(), &JsValue::from_str(&format!("{e:?}"))),
                        }
                        return;
                    }

                    spawn_local(async move {
                        match WebGpuRenderer::new(canvas.clone()).await {
                            Ok(renderer) => {
                                ECS.with(|ecs| {
                                    if let Some(app) = &mut *ecs.borrow_mut() {
                                        let boxed: Box<dyn RenderPort> = Box::new(renderer);
                                        app.set_renderer(boxed);
                                    }
                                });
                                announce_renderer("WebGPU");
                                console::log_1(&"WebGPU renderer set".into());
                            }
                            Err(_e) => {
                                // Fallback silencioso a Canvas2D
                                if let Ok(renderer) = Canvas2DRenderer::new(canvas) {
                                    console::log_1(&"Canvas2D fallback".into());
                                    ECS.with(|ecs| {
                                        if let Some(app) = &mut *ecs.borrow_mut() {
                                            let boxed: Box<dyn RenderPort> = Box::new(renderer);
                                            app.set_renderer(boxed);
                                        }
                                    });
                                    announce_renderer("Canvas2D");
                                } else {
                                    console::warn_1(&"Canvas2D fallback failed".into());
                                }
                            }
                        }
                    });
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static RAF_CB: std::cell::RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut()>>> = const { std::cell::RefCell::new(None) };
}

// No almacenamos el renderer en TLS; vivirá dentro del mundo ECS como recurso NonSend

#[cfg(target_arch = "wasm32")]
fn start_raf_loop() {
    RAF_CB.with(|cell| {
        // Create the closure and keep it alive inside thread_local
        let mut slot = cell.borrow_mut();
        if slot.is_some() {
            // already running
            return;
        }
        let cb = Closure::wrap(Box::new(move || {
            FRAME.with(|f| {
                let n = f.get().wrapping_add(1);
                f.set(n);
            });
            ECS.with(|ecs| {
                if let Some(app) = &mut *ecs.borrow_mut() {
                    // Actualizar tamaño del canvas y DPR siempre (independiente de WebGPU)
                    if let Some(win) = window() {
                        if let Some(doc) = win.document() {
                            if let Some(elem) = doc.get_element_by_id("main-canvas") {
                                if let Ok(canvas) = elem.dyn_into::<HtmlCanvasElement>() {
                                    let w = canvas.width().max(1);
                                    let h = canvas.height().max(1);
                                    app.set_canvas_size(w, h);
                                    app.set_canvas_dpr(win.device_pixel_ratio() as f32);
                                }
                            }
                        }
                    }
                    app.run_frame();
                }
            });
            if let Some(win) = window() {
                // schedule next frame with the same closure
                RAF_CB.with(|cell| {
                    if let Some(cb) = cell.borrow().as_ref() {
                        let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
                    }
                });
            }
        }) as Box<dyn FnMut()>);
        if let Some(win) = window() {
            let _ = win.request_animation_frame(cb.as_ref().unchecked_ref());
        }
        *slot = Some(cb);
        // do not forget; keep it in RAF_CB for the lifetime of the app
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_pointer_down(x: f32, y: f32) {
    console::log_1(&format!("ecs_pointer_down({}, {})", x, y).into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_pointer_down(x, y);
        }
    });
}

#[wasm_bindgen]
pub struct PointerDownResult {
    clicked_handle_type: Option<u8>,
    entity_selected: bool,
}

#[wasm_bindgen]
impl PointerDownResult {
    #[wasm_bindgen(getter)]
    pub fn clicked_handle_type(&self) -> Option<u8> {
        self.clicked_handle_type
    }
    
    #[wasm_bindgen(getter)]
    pub fn entity_selected(&self) -> bool {
        self.entity_selected
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_pointer_down_with_modifiers(x: f32, y: f32, ctrl_key: bool, shift_key: bool) -> PointerDownResult {
    console::log_1(&format!("ecs_pointer_down_with_modifiers({}, {}, ctrl={}, shift={})", x, y, ctrl_key, shift_key).into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            let result = app.send_pointer_down_with_modifiers(x, y, ctrl_key, shift_key);
            PointerDownResult {
                clicked_handle_type: result.clicked_handle_type,
                entity_selected: result.entity_selected,
            }
        } else {
            PointerDownResult {
                clicked_handle_type: None,
                entity_selected: false,
            }
        }
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_detect_handle_hover(x: f32, y: f32) -> JsValue {
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            if let Some(handle_type) = app.detect_handle_click(x, y) {
                JsValue::from_f64(handle_type as f64)
            } else {
                JsValue::NULL
            }
        } else {
            JsValue::NULL
        }
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_detect_shape_hover(x: f32, y: f32) -> JsValue {
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            // Usar la misma lógica que en pointer_down pero solo para detección
            let dpr = 1.0; // Simplificado para hover, podríamos mejorarlo después
            let click_x = x * dpr;
            let click_y = y * dpr;
            
            // Buscar si hay alguna entidad en esa posición
            for (id, transform, _style, shape) in &app.document().entities {
                // Crear transform escalado por DPR
                let transform_physical = momentum_core::model::Transform {
                    x: transform.x * dpr,
                    y: transform.y * dpr,
                    scale_x: transform.scale_x * dpr,
                    scale_y: transform.scale_y * dpr,
                    rotation: transform.rotation,
                };
                
                // Escalar shape por DPR
                let mut shape_physical = shape.clone();
                match shape_physical {
                    momentum_core::model::Shape::Rect { ref mut w, ref mut h } => {
                        *w *= dpr;
                        *h *= dpr;
                    }
                    momentum_core::model::Shape::Ellipse { ref mut rx, ref mut ry } => {
                        *rx *= dpr;
                        *ry *= dpr;
                    }
                    momentum_core::model::Shape::Line { ref mut x2, ref mut y2 } => {
                        *x2 *= dpr;
                        *y2 *= dpr;
                    }
                    momentum_core::model::Shape::Polygon { ref mut points } => {
                        for (px, py) in points {
                            *px *= dpr;
                            *py *= dpr;
                        }
                    }
                }
                
                // Usar sistema hitbox o fallback al shape
                if let Some(hitbox) = app.document().get_hitbox(*id) {
                    if hitbox.hit_test(click_x, click_y, &transform_physical, &shape_physical) {
                        return JsValue::from_bool(true);
                    }
                } else {
                    // Fallback: usar shape con tolerancia por defecto
                    let default_hitbox = momentum_core::model::Hitbox::from_shape(&shape_physical);
                    if default_hitbox.hit_test(click_x, click_y, &transform_physical, &shape_physical) {
                        return JsValue::from_bool(true);
                    }
                }
            }
            JsValue::from_bool(false)
        } else {
            JsValue::from_bool(false)
        }
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_create_rect(x: f32, y: f32, w: f32, h: f32) {
    console::log_1(&format!("ecs_create_rect(x={}, y={}, w={}, h={})", x, y, w, h).into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_create_rect(x, y, w, h);
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_create_ellipse(x: f32, y: f32, rx: f32, ry: f32) {
    console::log_1(&format!("ecs_create_ellipse(x={}, y={}, rx={}, ry={})", x, y, rx, ry).into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_create_ellipse(x, y, rx, ry);
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_create_line(x1: f32, y1: f32, x2: f32, y2: f32) {
    console::log_1(&format!("ecs_create_line(x1={}, y1={}, x2={}, y2={})", x1, y1, x2, y2).into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_create_line(x1, y1, x2, y2);
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_move_start(x: f32, y: f32) {
    console::log_1(&format!("ecs_move_start({}, {})", x, y).into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_move_start(x, y);
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_move_update(dx: f32, dy: f32) {
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_move_update(dx, dy);
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_move_end() {
    console::log_1(&"ecs_move_end()".into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_move_end();
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_scale_start(handle_type: u8, x: f32, y: f32) {
    use momentum_core::model::HandleType;
    let handle_type = match handle_type {
        0 => HandleType::TopLeft,
        1 => HandleType::TopRight,
        2 => HandleType::BottomLeft,
        3 => HandleType::BottomRight,
        4 => HandleType::Top,
        5 => HandleType::Right,
        6 => HandleType::Bottom,
        7 => HandleType::Left,
        _ => HandleType::TopLeft, // Default fallback
    };
    console::log_1(&format!("ecs_scale_start({:?}, {}, {})", handle_type, x, y).into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_scale_start(handle_type, x, y);
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_scale_update(dx: f32, dy: f32) {
    console::log_1(&format!("ecs_scale_update({}, {})", dx, dy).into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_scale_update(dx, dy);
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn ecs_scale_end() {
    console::log_1(&"ecs_scale_end()".into());
    ECS.with(|ecs| {
        if let Some(app) = &mut *ecs.borrow_mut() {
            app.send_scale_end();
        }
    });
}

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
struct RectDto { x: f32, y: f32, w: f32, h: f32 }

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_namespace = window)]
pub fn get_document_json() -> String {
    ECS.with(|ecs| {
        if let Some(app) = &*ecs.borrow() {
            let doc = app.document();
            let mut rects: Vec<RectDto> = Vec::with_capacity(doc.entities.len());
            for (_id, t, _s, shape) in &doc.entities {
                if let Shape::Rect { w, h } = shape {
                    rects.push(RectDto { x: t.x, y: t.y, w: *w, h: *h });
                }
            }
            let s = serde_json::to_string(&rects).unwrap_or_else(|_| "[]".to_string());
            console::log_1(&format!("get_document_json -> len={}, count={}", s.len(), rects.len()).into());
            s
        } else {
            "[]".to_string()
        }
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Mostrar panics de Rust en la consola del navegador
    console_error_panic_hook::set_once();
    console::log_1(&"Hodei Momentum — WASM start".into());
    // Inicializar ECS
    ECS.with(|ecs| {
        *ecs.borrow_mut() = Some(MomentumEcsApp::new());
    });

    // Registrar funciones globales en globalThis
    console::log_1(&"Before register_window_functions".into());
    if let Err(e) = register_window_functions() {
        console::log_2(&"register_window_functions ERROR".into(), &e);
    } else {
        console::log_1(&"register_window_functions OK".into());
    }

    // Montar UI de Leptos
    mount_app();

    // Intentar inicializar un renderer: WebGPU si está disponible, si no Canvas2D como fallback
    #[cfg(target_arch = "wasm32")]
    {
        let canvas = web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.get_element_by_id("main-canvas"))
            .and_then(|elem| elem.dyn_into::<HtmlCanvasElement>().ok())
            .expect("Failed to get canvas element");

        #[cfg(all(target_arch = "wasm32", feature = "webgpu"))]
        {
            // Si navigator.gpu no existe, no intentamos WebGPU y vamos directo a Canvas2D para evitar ruido en consola
            let has_webgpu = js_sys::Reflect::get(
                &web_sys::window().unwrap().navigator().into(),
                &JsValue::from_str("gpu"),
            )
            .ok()
            .map(|v| !v.is_undefined() && !v.is_null())
            .unwrap_or(false);

            if has_webgpu {
                // Try WebGPU first
                spawn_local(async move {
                    match WebGpuRenderer::new(canvas.clone()).await {
                        Ok(renderer) => {
                            console::log_1(&"WebGPU inicializado".into());
                            ECS.with(|ecs| {
                                if let Some(app) = &mut *ecs.borrow_mut() {
                                    let boxed: Box<dyn RenderPort> = Box::new(renderer);
                                    app.set_renderer(boxed);
                                }
                            });
                        }
                        Err(_e) => {
                            // Fallback silencioso a Canvas2D
                            if let Ok(renderer) = Canvas2DRenderer::new(canvas) {
                                console::log_1(&"Canvas2D fallback".into());
                                ECS.with(|ecs| {
                                    if let Some(app) = &mut *ecs.borrow_mut() {
                                        let boxed: Box<dyn RenderPort> = Box::new(renderer);
                                        app.set_renderer(boxed);
                                    }
                                });
                            } else {
                                console::warn_1(&"Canvas2D fallback failed".into());
                            }
                        }
                    }
                });
            } else {
                // Sin WebGPU: inicializar Canvas2D directamente
                match Canvas2DRenderer::new(canvas) {
                    Ok(renderer) => {
                        console::log_1(&"Canvas2D inicializado".into());
                        ECS.with(|ecs| {
                            if let Some(app) = &mut *ecs.borrow_mut() {
                                let boxed: Box<dyn RenderPort> = Box::new(renderer);
                                app.set_renderer(boxed);
                            }
                        });
                        announce_renderer("Canvas2D");
                    }
                    Err(_) => console::warn_1(&"Canvas2D initialization failed".into()),
                }
            }
        }

        #[cfg(all(target_arch = "wasm32", not(feature = "webgpu")))]
        {
            // Directly use Canvas2D when WebGPU feature is off
            match Canvas2DRenderer::new(canvas) {
                Ok(renderer) => {
                    console::log_1(&"Canvas2D inicializado".into());
                    ECS.with(|ecs| {
                        if let Some(app) = &mut *ecs.borrow_mut() {
                            let boxed: Box<dyn RenderPort> = Box::new(renderer);
                            app.set_renderer(boxed);
                        }
                    });
                }
                Err(_) => console::warn_1(&"Canvas2D initialization failed".into()),
            }
        }
    }

    // Bucle principal
    start_raf_loop();

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start() {}

#[cfg(target_arch = "wasm32")]
fn register_window_functions() -> Result<(), JsValue> {
    use js_sys::{global, Reflect};
    let global = global();
    let win_opt = window();

    // ecs_pointer_down(x, y)
    let f_down = Closure::wrap(Box::new(move |x: f32, y: f32| {
        console::log_1(&format!("[global] ecs_pointer_down({}, {})", x, y).into());
        ECS.with(|ecs| {
            match ecs.try_borrow_mut() {
                Ok(mut ecs_mut) => {
                    if let Some(app) = &mut *ecs_mut {
                        app.send_pointer_down(x, y);
                    }
                }
                Err(_) => {
                    console::warn_1(&"ECS is busy (pointer_down); retrying soon".into());
                    // Reintentar en el siguiente tick para no perder el evento
                    if let Some(win) = window() {
                        let cb = Closure::once_into_js(move || {
                            ECS.with(|ecs| {
                                if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                                    if let Some(app) = &mut *ecs_mut {
                                        app.send_pointer_down(x, y);
                                    }
                                }
                            });
                        });
                        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                            cb.as_ref().unchecked_ref(), 0,
                        );
                        // cb moved into JS via once_into_js, no need to forget
                    }
                }
            }
        });
    }) as Box<dyn FnMut(f32, f32)>);
    Reflect::set(&global, &JsValue::from_str("ecs_pointer_down"), f_down.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_pointer_down"), f_down.as_ref()); }
    f_down.forget();

    // ecs_create_rect(x, y, w, h)
    let f_rect = Closure::wrap(Box::new(move |x: f32, y: f32, w: f32, h: f32| {
        console::log_1(&format!("[global] ecs_create_rect({}, {}, {}, {})", x, y, w, h).into());
        ECS.with(|ecs| {
            match ecs.try_borrow_mut() {
                Ok(mut ecs_mut) => {
                    if let Some(app) = &mut *ecs_mut {
                        app.send_create_rect(x, y, w, h);
                    }
                }
                Err(_) => {
                    console::warn_1(&"ECS is busy (create_rect); retrying soon".into());
                    // Reintentar en el siguiente tick para no perder el evento
                    if let Some(win) = window() {
                        let cb = Closure::once_into_js(move || {
                            ECS.with(|ecs| {
                                if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                                    if let Some(app) = &mut *ecs_mut {
                                        app.send_create_rect(x, y, w, h);
                                    }
                                }
                            });
                        });
                        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                            cb.as_ref().unchecked_ref(), 0,
                        );
                    }
                }
            }
        });
    }) as Box<dyn FnMut(f32, f32, f32, f32)>);
    Reflect::set(&global, &JsValue::from_str("ecs_create_rect"), f_rect.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_create_rect"), f_rect.as_ref()); }
    f_rect.forget();

    // ecs_create_ellipse(x, y, rx, ry)
    let f_ellipse = Closure::wrap(Box::new(move |x: f32, y: f32, rx: f32, ry: f32| {
        console::log_1(&format!("[global] ecs_create_ellipse({}, {}, {}, {})", x, y, rx, ry).into());
        ECS.with(|ecs| {
            match ecs.try_borrow_mut() {
                Ok(mut ecs_mut) => {
                    if let Some(app) = &mut *ecs_mut {
                        app.send_create_ellipse(x, y, rx, ry);
                    }
                }
                Err(_) => {
                    console::warn_1(&"ECS is busy (create_ellipse); retrying soon".into());
                    if let Some(win) = window() {
                        let cb = Closure::once_into_js(move || {
                            ECS.with(|ecs| {
                                if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                                    if let Some(app) = &mut *ecs_mut {
                                        app.send_create_ellipse(x, y, rx, ry);
                                    }
                                }
                            });
                        });
                        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                            cb.as_ref().unchecked_ref(), 0,
                        );
                    }
                }
            }
        });
    }) as Box<dyn FnMut(f32, f32, f32, f32)>);
    Reflect::set(&global, &JsValue::from_str("ecs_create_ellipse"), f_ellipse.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_create_ellipse"), f_ellipse.as_ref()); }
    f_ellipse.forget();

    // ecs_create_line(x1, y1, x2, y2)
    let f_line = Closure::wrap(Box::new(move |x1: f32, y1: f32, x2: f32, y2: f32| {
        console::log_1(&format!("[global] ecs_create_line({}, {}, {}, {})", x1, y1, x2, y2).into());
        ECS.with(|ecs| {
            match ecs.try_borrow_mut() {
                Ok(mut ecs_mut) => {
                    if let Some(app) = &mut *ecs_mut {
                        app.send_create_line(x1, y1, x2, y2);
                    }
                }
                Err(_) => {
                    console::warn_1(&"ECS is busy (create_line); retrying soon".into());
                    if let Some(win) = window() {
                        let cb = Closure::once_into_js(move || {
                            ECS.with(|ecs| {
                                if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                                    if let Some(app) = &mut *ecs_mut {
                                        app.send_create_line(x1, y1, x2, y2);
                                    }
                                }
                            });
                        });
                        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                            cb.as_ref().unchecked_ref(), 0,
                        );
                    }
                }
            }
        });
    }) as Box<dyn FnMut(f32, f32, f32, f32)>);
    Reflect::set(&global, &JsValue::from_str("ecs_create_line"), f_line.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_create_line"), f_line.as_ref()); }
    f_line.forget();

    // get_document_json() -> String
    let f_get = Closure::wrap(Box::new(move || -> JsValue {
        let s = ECS.with(|ecs| {
            if let Some(app) = &*ecs.borrow() {
                let doc = app.document();
                let mut rects: Vec<RectDto> = Vec::with_capacity(doc.entities.len());
                for (_id, t, _s, shape) in &doc.entities {
                    if let Shape::Rect { w, h } = shape {
                        rects.push(RectDto { x: t.x, y: t.y, w: *w, h: *h });
                    }
                }
                serde_json::to_string(&rects).unwrap_or_else(|_| "[]".to_string())
            } else {
                "[]".to_string()
            }
        });
        console::log_1(&format!("[global] get_document_json len={}", s.len()).into());
        JsValue::from_str(&s)
    }) as Box<dyn FnMut() -> JsValue>);
    Reflect::set(&global, &JsValue::from_str("get_document_json"), f_get.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("get_document_json"), f_get.as_ref()); }
    f_get.forget();

    // ecs_pointer_down_with_modifiers(x, y, ctrl_key, shift_key)
    let f_down_mod = Closure::wrap(Box::new(move |x: f32, y: f32, ctrl_key: bool, shift_key: bool| {
        console::log_1(&format!("[global] ecs_pointer_down_with_modifiers({}, {}, {}, {})", x, y, ctrl_key, shift_key).into());
        ECS.with(|ecs| {
            match ecs.try_borrow_mut() {
                Ok(mut ecs_mut) => {
                    if let Some(app) = &mut *ecs_mut {
                        app.send_pointer_down_with_modifiers(x, y, ctrl_key, shift_key);
                    }
                }
                Err(_) => {
                    console::warn_1(&"ECS is busy (pointer_down_with_modifiers); retrying soon".into());
                    if let Some(win) = window() {
                        let cb = Closure::once_into_js(move || {
                            ECS.with(|ecs| {
                                if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                                    if let Some(app) = &mut *ecs_mut {
                                        app.send_pointer_down_with_modifiers(x, y, ctrl_key, shift_key);
                                    }
                                }
                            });
                        });
                        let _ = win.set_timeout_with_callback_and_timeout_and_arguments_0(
                            cb.as_ref().unchecked_ref(), 0,
                        );
                    }
                }
            }
        });
    }) as Box<dyn FnMut(f32, f32, bool, bool)>);
    Reflect::set(&global, &JsValue::from_str("ecs_pointer_down_with_modifiers"), f_down_mod.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_pointer_down_with_modifiers"), f_down_mod.as_ref()); }
    f_down_mod.forget();

    // ecs_move_start(x, y)
    let f_move_start = Closure::wrap(Box::new(move |x: f32, y: f32| {
        console::log_1(&format!("[global] ecs_move_start({}, {})", x, y).into());
        ECS.with(|ecs| {
            if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                if let Some(app) = &mut *ecs_mut {
                    app.send_move_start(x, y);
                }
            }
        });
    }) as Box<dyn FnMut(f32, f32)>);
    Reflect::set(&global, &JsValue::from_str("ecs_move_start"), f_move_start.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_move_start"), f_move_start.as_ref()); }
    f_move_start.forget();

    // ecs_move_update(dx, dy)
    let f_move_update = Closure::wrap(Box::new(move |dx: f32, dy: f32| {
        ECS.with(|ecs| {
            if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                if let Some(app) = &mut *ecs_mut {
                    app.send_move_update(dx, dy);
                }
            }
        });
    }) as Box<dyn FnMut(f32, f32)>);
    Reflect::set(&global, &JsValue::from_str("ecs_move_update"), f_move_update.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_move_update"), f_move_update.as_ref()); }
    f_move_update.forget();

    // ecs_move_end()
    let f_move_end = Closure::wrap(Box::new(move || {
        console::log_1(&"[global] ecs_move_end()".into());
        ECS.with(|ecs| {
            if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                if let Some(app) = &mut *ecs_mut {
                    app.send_move_end();
                }
            }
        });
    }) as Box<dyn FnMut()>);
    Reflect::set(&global, &JsValue::from_str("ecs_move_end"), f_move_end.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_move_end"), f_move_end.as_ref()); }
    f_move_end.forget();

    // ecs_scale_start(handle_type, x, y)
    let f_scale_start = Closure::wrap(Box::new(move |handle_type: u8, x: f32, y: f32| {
        use momentum_core::model::HandleType;
        let handle = match handle_type {
            0 => HandleType::TopLeft,
            1 => HandleType::TopRight,
            2 => HandleType::BottomLeft,
            3 => HandleType::BottomRight,
            4 => HandleType::Top,
            5 => HandleType::Right,
            6 => HandleType::Bottom,
            7 => HandleType::Left,
            _ => HandleType::TopLeft,
        };
        console::log_1(&format!("[global] ecs_scale_start({:?}, {}, {})", handle, x, y).into());
        ECS.with(|ecs| {
            if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                if let Some(app) = &mut *ecs_mut {
                    app.send_scale_start(handle, x, y);
                }
            }
        });
    }) as Box<dyn FnMut(u8, f32, f32)>);
    Reflect::set(&global, &JsValue::from_str("ecs_scale_start"), f_scale_start.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_scale_start"), f_scale_start.as_ref()); }
    f_scale_start.forget();

    // ecs_scale_update(dx, dy)
    let f_scale_update = Closure::wrap(Box::new(move |dx: f32, dy: f32| {
        console::log_1(&format!("[global] ecs_scale_update({}, {})", dx, dy).into());
        ECS.with(|ecs| {
            if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                if let Some(app) = &mut *ecs_mut {
                    app.send_scale_update(dx, dy);
                }
            }
        });
    }) as Box<dyn FnMut(f32, f32)>);
    Reflect::set(&global, &JsValue::from_str("ecs_scale_update"), f_scale_update.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_scale_update"), f_scale_update.as_ref()); }
    f_scale_update.forget();

    // ecs_scale_end()
    let f_scale_end = Closure::wrap(Box::new(move || {
        console::log_1(&"[global] ecs_scale_end()".into());
        ECS.with(|ecs| {
            if let Ok(mut ecs_mut) = ecs.try_borrow_mut() {
                if let Some(app) = &mut *ecs_mut {
                    app.send_scale_end();
                }
            }
        });
    }) as Box<dyn FnMut()>);
    Reflect::set(&global, &JsValue::from_str("ecs_scale_end"), f_scale_end.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_scale_end"), f_scale_end.as_ref()); }
    f_scale_end.forget();

    // ecs_detect_handle_hover
    let f_detect_handle_hover = Closure::wrap(Box::new(move |x: f32, y: f32| -> JsValue {
        ECS.with(|ecs| {
            if let Some(app) = &mut *ecs.borrow_mut() {
                if let Some(handle_type) = app.detect_handle_click(x, y) {
                    JsValue::from_f64(handle_type as f64)
                } else {
                    JsValue::NULL
                }
            } else {
                JsValue::NULL
            }
        })
    }) as Box<dyn FnMut(f32, f32) -> JsValue>);
    Reflect::set(&global, &JsValue::from_str("ecs_detect_handle_hover"), f_detect_handle_hover.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_detect_handle_hover"), f_detect_handle_hover.as_ref()); }
    f_detect_handle_hover.forget();

    // ecs_detect_shape_hover
    let f_detect_shape_hover = Closure::wrap(Box::new(move |x: f32, y: f32| -> JsValue {
        ECS.with(|ecs| {
            if let Some(app) = &mut *ecs.borrow_mut() {
                // Usar la misma lógica que en pointer_down pero solo para detección
                let dpr = 1.0; // Simplificado para hover
                let click_x = x * dpr;
                let click_y = y * dpr;
                
                // Buscar si hay alguna entidad en esa posición
                for (id, transform, _style, shape) in &app.document().entities {
                    // Crear transform escalado por DPR
                    let transform_physical = momentum_core::model::Transform {
                        x: transform.x * dpr,
                        y: transform.y * dpr,
                        scale_x: transform.scale_x * dpr,
                        scale_y: transform.scale_y * dpr,
                        rotation: transform.rotation,
                    };
                    
                    // Escalar shape por DPR
                    let mut shape_physical = shape.clone();
                    match shape_physical {
                        momentum_core::model::Shape::Rect { ref mut w, ref mut h } => {
                            *w *= dpr;
                            *h *= dpr;
                        }
                        momentum_core::model::Shape::Ellipse { ref mut rx, ref mut ry } => {
                            *rx *= dpr;
                            *ry *= dpr;
                        }
                        momentum_core::model::Shape::Line { ref mut x2, ref mut y2 } => {
                            *x2 *= dpr;
                            *y2 *= dpr;
                        }
                        momentum_core::model::Shape::Polygon { ref mut points } => {
                            for (px, py) in points {
                                *px *= dpr;
                                *py *= dpr;
                            }
                        }
                    }
                    
                    // Usar sistema hitbox o fallback al shape
                    if let Some(hitbox) = app.document().get_hitbox(*id) {
                        if hitbox.hit_test(click_x, click_y, &transform_physical, &shape_physical) {
                            return JsValue::from_bool(true);
                        }
                    } else {
                        // Fallback: usar shape con tolerancia por defecto
                        let default_hitbox = momentum_core::model::Hitbox::from_shape(&shape_physical);
                        if default_hitbox.hit_test(click_x, click_y, &transform_physical, &shape_physical) {
                            return JsValue::from_bool(true);
                        }
                    }
                }
                JsValue::from_bool(false)
            } else {
                JsValue::from_bool(false)
            }
        })
    }) as Box<dyn FnMut(f32, f32) -> JsValue>);
    Reflect::set(&global, &JsValue::from_str("ecs_detect_shape_hover"), f_detect_shape_hover.as_ref())?;
    if let Some(win) = &win_opt { let _ = Reflect::set(win, &JsValue::from_str("ecs_detect_shape_hover"), f_detect_shape_hover.as_ref()); }
    f_detect_shape_hover.forget();

    console::log_1(&"Funciones globales registradas en globalThis/window".into());
    Ok(())
}
