#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use momentum_core::model::{Color, Shape, Transform};
use momentum_core::ports::{RenderError, RenderPort};

pub struct Canvas2DRenderer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

impl Canvas2DRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, RenderError> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| RenderError::Initialization)?
            .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
            .ok_or(RenderError::Initialization)?;
        Ok(Self { canvas, ctx })
    }
}

impl RenderPort for Canvas2DRenderer {
    fn begin_frame(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        // Ensure canvas size matches requested physical size
        if self.canvas.width() != width { self.canvas.set_width(width); }
        if self.canvas.height() != height { self.canvas.set_height(height); }
        // Clear
        self.ctx.set_global_alpha(1.0);
        self.ctx.set_fill_style(&"#ffffff".into());
        let w = width as f64;
        let h = height as f64;
        self.ctx.fill_rect(0.0, 0.0, w, h);
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), RenderError> { Ok(()) }

    fn set_camera(&mut self, _transform_2d: [f32; 6]) -> Result<(), RenderError> { Ok(()) }

    fn draw_shape(&mut self, transform: &Transform, shape: &Shape, style: &momentum_core::model::Style) -> Result<(), RenderError> {
        match shape {
            Shape::Rect { w, h } => {
                // Stroke only for now
                if let Some(Color(r, g, b, a)) = style.stroke {
                    self.ctx.set_line_width(style.stroke_width as f64);
                    self.ctx.set_stroke_style(&format!("rgba({},{},{},{})", (r*255.0) as u32, (g*255.0) as u32, (b*255.0) as u32, a * style.opacity).into());
                    // Dashes
                    if !style.dash.is_empty() {
                        let arr = js_sys::Array::new();
                        for d in &style.dash { arr.push(&JsValue::from_f64(*d as f64)); }
                        let _ = self.ctx.set_line_dash(&arr);
                        self.ctx.set_line_dash_offset(style.dash_offset as f64);
                    } else {
                        let _ = self.ctx.set_line_dash(&js_sys::Array::new());
                        self.ctx.set_line_dash_offset(0.0);
                    }
                    let x = transform.x as f64;
                    let y = transform.y as f64;
                    self.ctx.stroke_rect(x, y, *w as f64, *h as f64);
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn draw_path(&mut self, _transform: &Transform, _path: &momentum_core::model::Path, _style: &momentum_core::model::Style) -> Result<(), RenderError> { Ok(()) }

    fn draw_text(&mut self, _transform: &Transform, _span: &momentum_core::model::TextSpan) -> Result<(), RenderError> { Ok(()) }

    fn measure_text(&mut self, _span: &momentum_core::model::TextSpan) -> Result<momentum_core::model::TextMetrics, RenderError> {
        Err(RenderError::Unsupported)
    }

    fn upload_image(&mut self, _id: momentum_core::model::ImageId, _data: &[u8]) -> Result<(), RenderError> { Ok(()) }

    fn draw_image(&mut self, _id: momentum_core::model::ImageId, _dest: momentum_core::model::Rect, _transform: &Transform, _tint: Option<momentum_core::model::Color>) -> Result<(), RenderError> { Ok(()) }
}
