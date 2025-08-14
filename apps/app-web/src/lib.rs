#[cfg(target_arch = "wasm32")]
use momentum_ecs::MomentumEcsApp;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{console, window};
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
                    spawn_local(async move {
                        match WebGpuRenderer::new(canvas).await {
                            Ok(renderer) => {
                                ECS.with(|ecs| {
                                    if let Some(app) = &mut *ecs.borrow_mut() {
                                        let boxed: Box<dyn RenderPort> = Box::new(renderer);
                                        app.set_renderer(boxed);
                                    }
                                });
                                console::log_1(&"WebGPU renderer set".into());
                            }
                            Err(e) => console::log_2(&"WebGPU init error".into(), &JsValue::from_str(&format!("{e:?}"))),
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
                console::log_1(&format!("raf_frame #{}", n).into());
            });
            ECS.with(|ecs| {
                if let Some(app) = &mut *ecs.borrow_mut() {
                    console::log_1(&"ecs.run_frame()".into());
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
                    // Debug: log document entity count to verify creations even without renderer
                    let count = app.document().count();
                    console::log_1(&format!("document entities = {}", count).into());
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
pub async fn start() -> Result<(), JsValue> {
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
                    Err(e) => {
                        console::log_2(&"WebGPU init error".into(), &JsValue::from_str(&format!("{e:?}")));
                        // Fallback to Canvas2D
                        if let Ok(renderer) = Canvas2DRenderer::new(canvas) {
                            console::log_1(&"Canvas2D fallback inicializado".into());
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

    console::log_1(&"Funciones globales registradas en globalThis/window".into());
    Ok(())
}
