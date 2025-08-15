#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};
#[cfg(target_arch = "wasm32")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use momentum_core::model::{Color, Shape, Transform};
use momentum_core::ports::{RenderError, RenderPort};

pub struct Canvas2DRenderer {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    // Store camera transform matrix [a, b, c, d, e, f] for 2D affine transformation
    // Represents: [scaleX, skewY, skewX, scaleY, translateX, translateY]
    camera_transform: [f32; 6],
}

impl Canvas2DRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, RenderError> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| RenderError::Initialization)?
            .and_then(|c| c.dyn_into::<CanvasRenderingContext2d>().ok())
            .ok_or(RenderError::Initialization)?;
        
        // Initialize with identity matrix (no transformation)
        let camera_transform = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        
        Ok(Self { canvas, ctx, camera_transform })
    }
}

impl RenderPort for Canvas2DRenderer {
    fn begin_frame(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        // Ensure canvas size matches requested physical size
        if self.canvas.width() != width { self.canvas.set_width(width); }
        if self.canvas.height() != height { self.canvas.set_height(height); }
        
        // Save the initial context state
        self.ctx.save();
        
        // Reset transformation matrix to identity
        self.ctx.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
            .map_err(|_| RenderError::Other("Failed to reset transform".into()))?;
        
        // Clear the entire canvas
        self.ctx.set_global_alpha(1.0);
        self.ctx.set_fill_style(&"#ffffff".into());
        let w = width as f64;
        let h = height as f64;
        self.ctx.fill_rect(0.0, 0.0, w, h);
        
        // Apply camera transformation
        let [a, b, c, d, e, f] = self.camera_transform;
        self.ctx.set_transform(
            a as f64, b as f64, c as f64, d as f64, e as f64, f as f64
        ).map_err(|_| RenderError::Other("Failed to set camera transform".into()))?;
        
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), RenderError> {
        // Restore the initial context state
        self.ctx.restore();
        Ok(())
    }

    fn set_camera(&mut self, transform_2d: [f32; 6]) -> Result<(), RenderError> {
        // Store the camera transformation matrix
        // The matrix represents [scaleX, skewY, skewX, scaleY, translateX, translateY]
        self.camera_transform = transform_2d;
        Ok(())
    }

    fn draw_shape(&mut self, transform: &Transform, shape: &Shape, style: &momentum_core::model::Style) -> Result<(), RenderError> {
        // Save context for transformations
        self.ctx.save();
        
        // Apply transform (translation, rotation, scale)
        let x = transform.x as f64;
        let y = transform.y as f64;
        let rotation = transform.rotation as f64;
        let scale_x = transform.scale_x as f64;
        let scale_y = transform.scale_y as f64;
        
        // Translate to position
        self.ctx.translate(x, y).map_err(|_| RenderError::Other("Translation failed".into()))?;
        
        // Apply rotation if any
        if rotation != 0.0 {
            self.ctx.rotate(rotation).map_err(|_| RenderError::Other("Rotation failed".into()))?;
        }
        
        // Apply scale if not default
        if scale_x != 1.0 || scale_y != 1.0 {
            self.ctx.scale(scale_x, scale_y).map_err(|_| RenderError::Other("Scale failed".into()))?;
        }
        
        // Setup line style for strokes
        if style.stroke.is_some() {
            self.ctx.set_line_width(style.stroke_width as f64);
            
            // Set stroke cap
            let cap = match style.stroke_cap {
                momentum_core::model::StrokeCap::Butt => "butt",
                momentum_core::model::StrokeCap::Square => "square", 
                momentum_core::model::StrokeCap::Round => "round",
            };
            self.ctx.set_line_cap(cap);
            
            // Set stroke join
            let join = match style.stroke_join {
                momentum_core::model::StrokeJoin::Miter => "miter",
                momentum_core::model::StrokeJoin::Bevel => "bevel",
                momentum_core::model::StrokeJoin::Round => "round",
            };
            self.ctx.set_line_join(join);
            
            // Set dash pattern
            if !style.dash.is_empty() {
                let arr = js_sys::Array::new();
                for d in &style.dash { 
                    arr.push(&JsValue::from_f64(*d as f64)); 
                }
                let _ = self.ctx.set_line_dash(&arr);
                self.ctx.set_line_dash_offset(style.dash_offset as f64);
            } else {
                let _ = self.ctx.set_line_dash(&js_sys::Array::new());
                self.ctx.set_line_dash_offset(0.0);
            }
        }
        
        match shape {
            Shape::Rect { w, h } => {
                let width = *w as f64;
                let height = *h as f64;
                
                // Fill if specified
                if let Some(Color(r, g, b, a)) = style.fill {
                    self.ctx.set_fill_style(&format!("rgba({},{},{},{})", 
                        (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, 
                        a * style.opacity).into());
                    self.ctx.fill_rect(0.0, 0.0, width, height);
                }
                
                // Stroke if specified
                if let Some(Color(r, g, b, a)) = style.stroke {
                    self.ctx.set_stroke_style(&format!("rgba({},{},{},{})", 
                        (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, 
                        a * style.opacity).into());
                    self.ctx.stroke_rect(0.0, 0.0, width, height);
                }
            }
            
            Shape::Ellipse { rx, ry } => {
                let rx_d = *rx as f64;
                let ry_d = *ry as f64;
                
                self.ctx.begin_path();
                // Draw ellipse using scale transformation
                if rx_d != ry_d {
                    self.ctx.save();
                    self.ctx.scale(1.0, ry_d / rx_d).map_err(|_| RenderError::Other("Ellipse scale failed".into()))?;
                    self.ctx.arc(0.0, 0.0, rx_d, 0.0, 2.0 * std::f64::consts::PI).map_err(|_| RenderError::Other("Arc failed".into()))?;
                    self.ctx.restore();
                } else {
                    // Perfect circle
                    self.ctx.arc(0.0, 0.0, rx_d, 0.0, 2.0 * std::f64::consts::PI).map_err(|_| RenderError::Other("Arc failed".into()))?;
                }
                
                // Fill if specified
                if let Some(Color(r, g, b, a)) = style.fill {
                    self.ctx.set_fill_style(&format!("rgba({},{},{},{})", 
                        (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, 
                        a * style.opacity).into());
                    self.ctx.fill();
                }
                
                // Stroke if specified
                if let Some(Color(r, g, b, a)) = style.stroke {
                    self.ctx.set_stroke_style(&format!("rgba({},{},{},{})", 
                        (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, 
                        a * style.opacity).into());
                    self.ctx.stroke();
                }
            }
            
            Shape::Line { x2, y2 } => {
                self.ctx.begin_path();
                self.ctx.move_to(0.0, 0.0);
                self.ctx.line_to(*x2 as f64, *y2 as f64);
                
                // Lines only support stroke
                if let Some(Color(r, g, b, a)) = style.stroke {
                    self.ctx.set_stroke_style(&format!("rgba({},{},{},{})", 
                        (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, 
                        a * style.opacity).into());
                    self.ctx.stroke();
                }
            }
            
            Shape::Polygon { points } => {
                if points.is_empty() {
                    // Nothing to draw
                } else {
                    self.ctx.begin_path();
                    
                    // Move to first point
                    let first = &points[0];
                    self.ctx.move_to(first.0 as f64, first.1 as f64);
                    
                    // Line to remaining points
                    for point in points.iter().skip(1) {
                        self.ctx.line_to(point.0 as f64, point.1 as f64);
                    }
                    
                    // Close the path
                    self.ctx.close_path();
                    
                    // Fill if specified
                    if let Some(Color(r, g, b, a)) = style.fill {
                        self.ctx.set_fill_style(&format!("rgba({},{},{},{})", 
                            (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, 
                            a * style.opacity).into());
                        self.ctx.fill();
                    }
                    
                    // Stroke if specified
                    if let Some(Color(r, g, b, a)) = style.stroke {
                        self.ctx.set_stroke_style(&format!("rgba({},{},{},{})", 
                            (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, 
                            a * style.opacity).into());
                        self.ctx.stroke();
                    }
                }
            }
        }
        
        // Restore context
        self.ctx.restore();
        
        Ok(())
    }

    fn draw_path(&mut self, transform: &Transform, path: &momentum_core::model::Path, style: &momentum_core::model::Style) -> Result<(), RenderError> {
        if path.commands.is_empty() {
            return Ok(());
        }
        
        // Save context for transformations
        self.ctx.save();
        
        // Apply transform (translation, rotation, scale)
        let x = transform.x as f64;
        let y = transform.y as f64;
        let rotation = transform.rotation as f64;
        let scale_x = transform.scale_x as f64;
        let scale_y = transform.scale_y as f64;
        
        // Translate to position
        self.ctx.translate(x, y).map_err(|_| RenderError::Other("Translation failed".into()))?;
        
        // Apply rotation if any
        if rotation != 0.0 {
            self.ctx.rotate(rotation).map_err(|_| RenderError::Other("Rotation failed".into()))?;
        }
        
        // Apply scale if not default
        if scale_x != 1.0 || scale_y != 1.0 {
            self.ctx.scale(scale_x, scale_y).map_err(|_| RenderError::Other("Scale failed".into()))?;
        }
        
        // Setup line style for strokes
        if style.stroke.is_some() {
            self.ctx.set_line_width(style.stroke_width as f64);
            
            // Set stroke cap
            let cap = match style.stroke_cap {
                momentum_core::model::StrokeCap::Butt => "butt",
                momentum_core::model::StrokeCap::Square => "square", 
                momentum_core::model::StrokeCap::Round => "round",
            };
            self.ctx.set_line_cap(cap);
            
            // Set stroke join
            let join = match style.stroke_join {
                momentum_core::model::StrokeJoin::Miter => "miter",
                momentum_core::model::StrokeJoin::Bevel => "bevel",
                momentum_core::model::StrokeJoin::Round => "round",
            };
            self.ctx.set_line_join(join);
            
            // Set dash pattern
            if !style.dash.is_empty() {
                let arr = js_sys::Array::new();
                for d in &style.dash { 
                    arr.push(&JsValue::from_f64(*d as f64)); 
                }
                let _ = self.ctx.set_line_dash(&arr);
                self.ctx.set_line_dash_offset(style.dash_offset as f64);
            } else {
                let _ = self.ctx.set_line_dash(&js_sys::Array::new());
                self.ctx.set_line_dash_offset(0.0);
            }
        }
        
        // Begin path and execute commands
        self.ctx.begin_path();
        
        for command in &path.commands {
            match command {
                momentum_core::model::PathCommand::MoveTo(x, y) => {
                    self.ctx.move_to(*x as f64, *y as f64);
                }
                momentum_core::model::PathCommand::LineTo(x, y) => {
                    self.ctx.line_to(*x as f64, *y as f64);
                }
                momentum_core::model::PathCommand::QuadTo { cx, cy, x, y } => {
                    self.ctx.quadratic_curve_to(*cx as f64, *cy as f64, *x as f64, *y as f64);
                }
                momentum_core::model::PathCommand::CubicTo { c1x, c1y, c2x, c2y, x, y } => {
                    self.ctx.bezier_curve_to(
                        *c1x as f64, *c1y as f64,
                        *c2x as f64, *c2y as f64,
                        *x as f64, *y as f64
                    );
                }
                momentum_core::model::PathCommand::Close => {
                    self.ctx.close_path();
                }
            }
        }
        
        // Fill if specified
        if let Some(Color(r, g, b, a)) = style.fill {
            self.ctx.set_fill_style(&format!("rgba({},{},{},{})", 
                (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, 
                a * style.opacity).into());
            self.ctx.fill();
        }
        
        // Stroke if specified
        if let Some(Color(r, g, b, a)) = style.stroke {
            self.ctx.set_stroke_style(&format!("rgba({},{},{},{})", 
                (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, 
                a * style.opacity).into());
            self.ctx.stroke();
        }
        
        // Restore context
        self.ctx.restore();
        
        Ok(())
    }

    fn draw_text(&mut self, transform: &Transform, span: &momentum_core::model::TextSpan) -> Result<(), RenderError> {
        if span.text.is_empty() {
            return Ok(());
        }
        
        // Save context for transformations
        self.ctx.save();
        
        // Apply transform (translation, rotation, scale)
        let x = transform.x as f64;
        let y = transform.y as f64;
        let rotation = transform.rotation as f64;
        let scale_x = transform.scale_x as f64;
        let scale_y = transform.scale_y as f64;
        
        // Translate to position
        self.ctx.translate(x, y).map_err(|_| RenderError::Other("Translation failed".into()))?;
        
        // Apply rotation if any
        if rotation != 0.0 {
            self.ctx.rotate(rotation).map_err(|_| RenderError::Other("Rotation failed".into()))?;
        }
        
        // Apply scale if not default
        if scale_x != 1.0 || scale_y != 1.0 {
            self.ctx.scale(scale_x, scale_y).map_err(|_| RenderError::Other("Scale failed".into()))?;
        }
        
        // Setup font
        let font_family = span.font_family.as_deref().unwrap_or("system-ui, sans-serif");
        let weight = span.weight.unwrap_or(400);
        let font_string = format!("{} {}px {}", weight, span.size, font_family);
        self.ctx.set_font(&font_string);
        
        // Set text baseline and alignment
        self.ctx.set_text_baseline("top");
        self.ctx.set_text_align("start");
        
        // Setup color
        let Color(r, g, b, a) = span.color;
        let color_string = format!("rgba({},{},{},{})", 
            (r * 255.0) as u32, (g * 255.0) as u32, (b * 255.0) as u32, a);
        
        // Draw text (we use fill_text as it's more common for text rendering)
        self.ctx.set_fill_style(&color_string.into());
        self.ctx.fill_text(&span.text, 0.0, 0.0).map_err(|_| RenderError::TextShaping)?;
        
        // Restore context
        self.ctx.restore();
        
        Ok(())
    }

    fn measure_text(&mut self, span: &momentum_core::model::TextSpan) -> Result<momentum_core::model::TextMetrics, RenderError> {
        if span.text.is_empty() {
            return Ok(momentum_core::model::TextMetrics {
                width: 0.0,
                ascent: 0.0,
                descent: 0.0,
                line_gap: 0.0,
            });
        }
        
        // Setup font for measurement
        let font_family = span.font_family.as_deref().unwrap_or("system-ui, sans-serif");
        let weight = span.weight.unwrap_or(400);
        let font_string = format!("{} {}px {}", weight, span.size, font_family);
        self.ctx.set_font(&font_string);
        
        // Simple text width estimation since Canvas2D measure_text might not be available
        // This is a rough approximation based on average character width
        let avg_char_width = span.size * 0.6; // Rough approximation for typical fonts
        let width = span.text.len() as f32 * avg_char_width;
        
        // Estimate ascent and descent based on font size
        // These are approximations since Canvas2D doesn't provide exact font metrics
        let ascent = span.size * 0.8; // Rough approximation
        let descent = span.size * 0.2; // Rough approximation  
        let line_gap = span.size * 0.1; // Small gap
        
        Ok(momentum_core::model::TextMetrics {
            width,
            ascent,
            descent,
            line_gap,
        })
    }

    fn upload_image(&mut self, _id: momentum_core::model::ImageId, _data: &[u8]) -> Result<(), RenderError> { Ok(()) }

    fn draw_image(&mut self, _id: momentum_core::model::ImageId, _dest: momentum_core::model::Rect, _transform: &Transform, _tint: Option<momentum_core::model::Color>) -> Result<(), RenderError> { Ok(()) }

    fn draw_scale_handle(&mut self, handle: &momentum_core::model::ScaleHandle) -> Result<(), RenderError> {
        self.ctx.save();
        
        // Dibujar el handle como un círculo blanco con borde azul (como Excalidraw)
        let center_x = handle.x as f64 + (handle.size as f64) / 2.0;
        let center_y = handle.y as f64 + (handle.size as f64) / 2.0;
        let radius = (handle.size as f64) / 2.0;
        
        self.ctx.begin_path();
        self.ctx.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI).map_err(|_| RenderError::Other("Arc failed".into()))?;
        
        // Fondo blanco
        self.ctx.set_fill_style(&"#ffffff".into());
        self.ctx.fill();
        
        // Borde azul
        self.ctx.set_stroke_style(&"#1971c2".into()); // Azul más oscuro como Excalidraw
        self.ctx.set_line_width(1.5);
        self.ctx.stroke();
        
        self.ctx.restore();
        Ok(())
    }
}
