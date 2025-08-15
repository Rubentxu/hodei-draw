#![cfg(all(target_arch = "wasm32", feature = "webgpu"))]

use std::{cell::RefCell, rc::Rc};
use web_sys::HtmlCanvasElement;
use wgpu::{InstanceDescriptor, PowerPreference, RequestAdapterOptions, SurfaceConfiguration, TextureUsages, PresentMode};
use wgpu::util::DeviceExt;

use momentum_core::ports::{RenderError, RenderPort};
use momentum_core::model::{Path, Rect, Shape, Style, TextMetrics, TextSpan, Transform, ImageId, ScaleHandle};

struct WgpuState {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_format: wgpu::TextureFormat,
    config: SurfaceConfiguration,
    canvas: HtmlCanvasElement,
    pipeline: wgpu::RenderPipeline,
    color_bgl: wgpu::BindGroupLayout,
    // Frame state for current begin_frame..end_frame
    cur_frame: Option<CurrentFrame>,
}

pub struct WebGpuRenderer {
    state: Rc<RefCell<WgpuState>>,
}

impl WebGpuRenderer {
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(&InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .map_err(|e| RenderError::Other(format!("No WebGPU adapter available: {e:?}")))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| RenderError::Other(format!("request_device error: {e}")))?;

        // Create a temporary surface to determine capabilities/format
        let tmp_surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|e| RenderError::Other(format!("create_surface (caps) error: {e:?}")))?;
        let surface_caps = tmp_surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        drop(tmp_surface);

        let width = canvas.width().max(1);
        let height = canvas.height().max(1);
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // Create a very simple pipeline: position-only, solid color fragment
        let shader_src = r#"
@group(0) @binding(0) var<uniform> u_color: vec4<f32>;
@vertex
fn vs_main(@location(0) a_pos: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(a_pos, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return u_color;
}
"#;
        let device_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("basic-rect-shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

        // Bind group layout for color uniform
        let color_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("color-bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(std::num::NonZeroU64::new(16).unwrap()),
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline-layout"),
            bind_group_layouts: &[&color_bgl],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rect-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &device_shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &device_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview: None,
            cache: None,
        });

        let state = Rc::new(RefCell::new(WgpuState { instance, adapter, device, queue, surface_format, config, canvas, pipeline, color_bgl, cur_frame: None }));
        Ok(Self { state })
    }
}

struct CurrentFrame {
    frame: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl RenderPort for WebGpuRenderer {
    fn begin_frame(&mut self, width: u32, height: u32) -> Result<(), RenderError> {
        let mut s = self.state.borrow_mut();
        if width != s.config.width || height != s.config.height {
            s.config.width = width.max(1);
            s.config.height = height.max(1);
        }
        let surface = s
            .instance
            .create_surface(wgpu::SurfaceTarget::Canvas(s.canvas.clone()))
            .map_err(|e| RenderError::Other(format!("create_surface (frame) error: {e:?}")))?;
        s.config.format = s.surface_format;
        surface.configure(&s.device, &s.config);
        let frame = surface
            .get_current_texture()
            .map_err(|e| RenderError::Other(format!("get_current_texture error: {e:?}")))?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = s.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("frame-encoder") });
        // First clear pass
        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.95, g: 0.96, b: 0.98, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }
        s.cur_frame = Some(CurrentFrame { frame, view, encoder });
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), RenderError> {
        let mut s = self.state.borrow_mut();
        if let Some(mut cur) = s.cur_frame.take() {
            s.queue.submit(std::iter::once(cur.encoder.finish()));
            cur.frame.present();
        }
        Ok(())
    }

    fn set_camera(&mut self, _transform_2d: [f32; 6]) -> Result<(), RenderError> { Ok(()) }

    fn draw_shape(&mut self, transform: &Transform, shape: &Shape, style: &Style) -> Result<(), RenderError> {
        // Only Rect supported for now
        let (w, h) = match shape {
            Shape::Rect { w, h } => (*w, *h),
            _ => return Ok(()),
        };
        let mut s = self.state.borrow_mut();
        // Gather immutable state before borrowing cur frame mutably
        let width = s.config.width.max(1) as f32;
        let height = s.config.height.max(1) as f32;
        let device = s.device.clone();
        let color_bgl = s.color_bgl.clone();
        let pipeline = s.pipeline.clone();
        let cur = match s.cur_frame.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };
        // top-left origin in pixels
        let x = transform.x;
        let y = transform.y;
        let xl = (x / width) * 2.0 - 1.0;
        let xr = ((x + w) / width) * 2.0 - 1.0;
        // flip Y axis: pixel y down -> NDC up
        let yt = 1.0 - (y / height) * 2.0;
        let yb = 1.0 - ((y + h) / height) * 2.0;
        let mut rpass = cur.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("rect-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &cur.view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rpass.set_pipeline(&pipeline);
        // Helper function to draw a quad given NDC bounds and RGBA color
        fn draw_quad_fn<'a>(
            rpass: &mut wgpu::RenderPass<'a>,
            device: &wgpu::Device,
            color_bgl: &wgpu::BindGroupLayout,
            xl: f32,
            yt: f32,
            xr: f32,
            yb: f32,
            color: [f32; 4],
        ) {
            let verts: [[f32; 2]; 6] = [
                [xl, yt], [xl, yb], [xr, yb],
                [xl, yt], [xr, yb], [xr, yt],
            ];
            let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("rect-vbuf"),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("color-ubuf"),
                contents: bytemuck::cast_slice(&color),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("color-bg"),
                layout: color_bgl,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() }],
            });
            rpass.set_bind_group(0, &bg, &[]);
            rpass.set_vertex_buffer(0, vbuf.slice(..));
            rpass.draw(0..6, 0..1);
        }

        // Fill pass
        if let Some(fill) = style.fill {
            let c = [fill.0, fill.1, fill.2, fill.3 * style.opacity.max(0.0).min(1.0)];
            draw_quad_fn(&mut rpass, &device, &color_bgl, xl, yt, xr, yb, c);
        }
        // Stroke pass: dashed support for rectangle edges. Caps: Butt / Square. Round treated as Butt for now.
        if let Some(st) = style.stroke {
            let a = st.3 * style.opacity.max(0.0).min(1.0);
            let color = [st.0, st.1, st.2, a];
            let sw = style.stroke_width.max(0.5);
            // Helper to draw a horizontal dashed edge at y in px from [x0,x1) with stroke sw
            fn draw_h_edge_fn(
                rpass: &mut wgpu::RenderPass<'_>,
                device: &wgpu::Device,
                color_bgl: &wgpu::BindGroupLayout,
                width: f32,
                height: f32,
                sw: f32,
                color: [f32;4],
                stroke_cap: momentum_core::model::StrokeCap,
                dash: &[f32],
                dash_offset: f32,
                x0_px: f32,
                x1_px: f32,
                y_px: f32,
            ) {
                let mut start = x0_px;
                // dash pattern handling
                if dash.is_empty() {
                    let xl_ = (start / width) * 2.0 - 1.0;
                    let xr_ = (x1_px / width) * 2.0 - 1.0;
                    let yt_ = 1.0 - (y_px / height) * 2.0;
                    let yb_ = 1.0 - ((y_px + sw) / height) * 2.0;
                    draw_quad_fn(rpass, device, color_bgl, xl_, yt_, xr_, yb_, color);
                    return;
                }
                // Normalized dash pattern cycle
                let mut pattern = dash.to_vec();
                if pattern.len() == 1 { pattern.push(pattern[0]); }
                let mut idx = 0usize;
                let mut on = true; // first segment on
                let mut rem = pattern[0];
                // apply dash_offset
                let mut offset = dash_offset.rem_euclid(pattern.iter().sum());
                while offset > 0.0 {
                    let step = rem.min(offset);
                    rem -= step;
                    offset -= step;
                    if rem <= 0.0 {
                        idx = (idx + 1) % pattern.len();
                        on = !on;
                        rem = pattern[idx];
                    }
                }
                while start < x1_px {
                    let seg = (start + rem).min(x1_px);
                    if on {
                        // cap adjustment for square caps: extend half sw at both ends
                        let mut sx = start;
                        let mut ex = seg;
                        if matches!(stroke_cap, momentum_core::model::StrokeCap::Square) {
                            sx -= sw * 0.5;
                            ex += sw * 0.5;
                        }
                        let xl_ = (sx / width) * 2.0 - 1.0;
                        let xr_ = (ex / width) * 2.0 - 1.0;
                        let yt_ = 1.0 - (y_px / height) * 2.0;
                        let yb_ = 1.0 - ((y_px + sw) / height) * 2.0;
                        draw_quad_fn(rpass, device, color_bgl, xl_, yt_, xr_, yb_, color);
                    }
                    start = seg;
                    rem -= seg - start; // zero
                    if rem <= 0.0 {
                        idx = (idx + 1) % pattern.len();
                        on = !on;
                        rem = pattern[idx];
                    }
                }
            }
            // Helper to draw a vertical dashed edge at x in px from [y0,y1) with stroke sw
            fn draw_v_edge_fn(
                rpass: &mut wgpu::RenderPass<'_>,
                device: &wgpu::Device,
                color_bgl: &wgpu::BindGroupLayout,
                width: f32,
                height: f32,
                sw: f32,
                color: [f32;4],
                stroke_cap: momentum_core::model::StrokeCap,
                dash: &[f32],
                dash_offset: f32,
                x_px: f32,
                y0_px: f32,
                y1_px: f32,
            ) {
                let mut start = y0_px;
                if dash.is_empty() {
                    let xl_ = (x_px / width) * 2.0 - 1.0;
                    let xr_ = ((x_px + sw) / width) * 2.0 - 1.0;
                    let yt_ = 1.0 - (start / height) * 2.0;
                    let yb_ = 1.0 - (y1_px / height) * 2.0;
                    draw_quad_fn(rpass, device, color_bgl, xl_, yt_, xr_, yb_, color);
                    return;
                }
                let mut pattern = dash.to_vec();
                if pattern.len() == 1 { pattern.push(pattern[0]); }
                let mut idx = 0usize;
                let mut on = true;
                let mut rem = pattern[0];
                let mut offset = dash_offset.rem_euclid(pattern.iter().sum());
                while offset > 0.0 {
                    let step = rem.min(offset);
                    rem -= step;
                    offset -= step;
                    if rem <= 0.0 {
                        idx = (idx + 1) % pattern.len();
                        on = !on;
                        rem = pattern[idx];
                    }
                }
                while start < y1_px {
                    let seg = (start + rem).min(y1_px);
                    if on {
                        let mut sy = start;
                        let mut ey = seg;
                        if matches!(stroke_cap, momentum_core::model::StrokeCap::Square) {
                            sy -= sw * 0.5;
                            ey += sw * 0.5;
                        }
                        let xl_ = (x_px / width) * 2.0 - 1.0;
                        let xr_ = ((x_px + sw) / width) * 2.0 - 1.0;
                        let yt_ = 1.0 - (sy / height) * 2.0;
                        let yb_ = 1.0 - (ey / height) * 2.0;
                        draw_quad_fn(rpass, device, color_bgl, xl_, yt_, xr_, yb_, color);
                    }
                    start = seg;
                    rem -= seg - start;
                    if rem <= 0.0 {
                        idx = (idx + 1) % pattern.len();
                        on = !on;
                        rem = pattern[idx];
                    }
                }
            }

            // Edges in pixel space
            // top: from (x,y) to (x+w,y)
            draw_h_edge_fn(&mut rpass, &device, &color_bgl, width, height, sw, color, style.stroke_cap, &style.dash, style.dash_offset, x, x + w, y);
            // bottom: at y+h-sw to y+h (align like previous implementation which used [yb+swy, yb])
            draw_h_edge_fn(&mut rpass, &device, &color_bgl, width, height, sw, color, style.stroke_cap, &style.dash, style.dash_offset, x, x + w, y + h - sw);
            // left: from (x,y) to (x,y+h)
            draw_v_edge_fn(&mut rpass, &device, &color_bgl, width, height, sw, color, style.stroke_cap, &style.dash, style.dash_offset, x, y, y + h);
            // right: at x+w-sw to x+w
            draw_v_edge_fn(&mut rpass, &device, &color_bgl, width, height, sw, color, style.stroke_cap, &style.dash, style.dash_offset, x + w - sw, y, y + h);
        }
        Ok(())
    }

    fn draw_path(&mut self, transform: &Transform, path: &Path, style: &Style) -> Result<(), RenderError> {
        let mut s = self.state.borrow_mut();
        if style.stroke.is_none() { return Ok(()); }
        let stroke = style.stroke.unwrap();
        let alpha = stroke.3 * style.opacity.max(0.0).min(1.0);
        let color = [stroke.0, stroke.1, stroke.2, alpha];
        let sw = style.stroke_width.max(0.5);
        // Gather immutable state before borrowing current frame mutably
        let width = s.config.width.max(1) as f32;
        let height = s.config.height.max(1) as f32;
        let device = s.device.clone();
        let color_bgl = s.color_bgl.clone();
        let pipeline = s.pipeline.clone();
        let cur = match s.cur_frame.as_mut() { Some(c) => c, None => return Ok(()) };

        // Render pass begun once for all segments
        let mut rpass = cur.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("path-stroke-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &cur.view,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rpass.set_pipeline(&pipeline);

        fn draw_tris_fn<'a>(
            rpass: &mut wgpu::RenderPass<'a>,
            device: &wgpu::Device,
            color_bgl: &wgpu::BindGroupLayout,
            color: [f32; 4],
            verts_ndc: &[[f32; 2]],
        ) {
            if verts_ndc.is_empty() { return; }
            let vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("tris-vbuf"),
                contents: bytemuck::cast_slice(verts_ndc),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let ubuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("color-ubuf"),
                contents: bytemuck::cast_slice(&color),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("color-bg"),
                layout: color_bgl,
                entries: &[wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() }],
            });
            rpass.set_bind_group(0, &bg, &[]);
            rpass.set_vertex_buffer(0, vbuf.slice(..));
            rpass.draw(0..(verts_ndc.len() as u32), 0..1);
        }

        let to_ndc = |px: f32, py: f32| -> [f32;2] {
            let x = (px / width) * 2.0 - 1.0;
            let y = 1.0 - (py / height) * 2.0;
            [x, y]
        };

        // Build polylines from path commands. Only MoveTo, LineTo, Close supported for stroke.
        let mut subpath: Vec<(f32,f32)> = Vec::new();
        let mut start_pt: Option<(f32,f32)> = None;
        let tx = transform.x; let ty = transform.y;
        let mut flush_subpath = |poly: &mut Vec<(f32,f32)>, _start: Option<(f32,f32)>, closed: bool| {
            if poly.len() < 2 { poly.clear(); return; }
            // Continuous dash state across the subpath
            let mut pattern = style.dash.clone();
            if pattern.len() == 1 { pattern.push(pattern[0]); }
            let use_dash = !pattern.is_empty();
            let mut pat_idx = 0usize;
            let mut on = true;
            let mut rem = if use_dash { pattern[0] } else { f32::INFINITY };
            // offset
            if use_dash {
                let mut offset = style.dash_offset.rem_euclid(pattern.iter().sum());
                while offset > 0.0 {
                    let step = rem.min(offset);
                    rem -= step;
                    offset -= step;
                    if rem <= 0.0 { pat_idx = (pat_idx + 1) % pattern.len(); on = !on; rem = pattern[pat_idx]; }
                }
            }

            let hw = sw * 0.5;
            let miter_limit = 4.0; // fallback to bevel if exceeded

            let pts = poly.clone();
            let seg_count = if closed { pts.len() } else { pts.len() - 1 };
            let get_pt = |i: usize| -> (f32,f32) { if i < pts.len() { pts[i] } else { pts[0] } };

            // draw each segment with dash splitting
            let mut prev_dir: Option<(f32,f32)> = None;
            let mut prev_norm: Option<(f32,f32)> = None;
            for si in 0..seg_count {
                let (mut x0, mut y0) = get_pt(si);
                let (mut x1, mut y1) = get_pt((si+1)%pts.len());
                // dash splitting along this segment, keeping continuous state
                let mut seg_len = ((x1-x0).hypot(y1-y0)).max(1e-6);
                let dir = ((x1-x0)/seg_len, (y1-y0)/seg_len);
                // square cap: extend ends for first/last visible piece
                let mut t0 = 0.0f32;
                while t0 < seg_len {
                    let take = rem.min(seg_len - t0);
                    if on {
                        let t1 = t0 + take;
                        // compute endpoints in px
                        let sx = x0 + dir.0 * t0;
                        let sy = y0 + dir.1 * t0;
                        let ex = x0 + dir.0 * t1;
                        let ey = y0 + dir.1 * t1;
                        // normal
                        let n = (-dir.1, dir.0);
                        // cap extend
                        let (mut csx, mut csy, mut cex, mut cey) = (sx, sy, ex, ey);
                        if matches!(style.stroke_cap, momentum_core::model::StrokeCap::Square) {
                            csx -= dir.0 * hw; csy -= dir.1 * hw;
                            cex += dir.0 * hw; cey += dir.1 * hw;
                        }
                        // segment quad
                        let v0 = to_ndc(csx + n.0*hw, csy + n.1*hw);
                        let v1 = to_ndc(csx - n.0*hw, csy - n.1*hw);
                        let v2 = to_ndc(cex - n.0*hw, cey - n.1*hw);
                        let v3 = to_ndc(cex + n.0*hw, cey + n.1*hw);
                        let tris = [v0, v1, v2, v0, v2, v3];
                        draw_tris_fn(&mut rpass, &device, &color_bgl, color, &tris);

                        // join with previous visible segment (bevel)
                        if t0 == 0.0 && si > 0 || (closed && si == 0 && poly.len() > 2) {
                            if let (Some(pd), Some(pn)) = (prev_dir, prev_norm) {
                                // Determine outer side
                                let cross = pd.0*dir.1 - pd.1*dir.0;
                                let outer_prev = if cross > 0.0 { (pn.0*hw, pn.1*hw) } else { (-pn.0*hw, -pn.1*hw) };
                                let outer_next = if cross > 0.0 { (n.0*hw, n.1*hw) } else { (-n.0*hw, -n.1*hw) };
                                let px = x0; let py = y0;
                                // miter candidate
                                if matches!(style.stroke_join, momentum_core::model::StrokeJoin::Miter) {
                                    // Compute intersection of offset lines
                                    let a1 = (px + outer_prev.0, py + outer_prev.1);
                                    let d1 = (pd.0, pd.1);
                                    let a2 = (px + outer_next.0, py + outer_next.1);
                                    let d2 = (dir.0, dir.1);
                                    let denom = d1.0*d2.1 - d1.1*d2.0;
                                    if denom.abs() > 1e-5 {
                                        let t = ((a2.0 - a1.0)*d2.1 - (a2.1 - a1.1)*d2.0) / denom;
                                        let mx = a1.0 + d1.0 * t;
                                        let my = a1.1 + d1.1 * t;
                                        let m_len = ((mx - px).hypot(my - py)) / hw;
                                        if m_len <= miter_limit {
                                            let p_outer_prev = to_ndc(px + outer_prev.0, py + outer_prev.1);
                                            let p_outer_next = to_ndc(px + outer_next.0, py + outer_next.1);
                                            let p_miter = to_ndc(mx, my);
                                            let tris2 = [p_outer_prev, p_miter, p_outer_next];
                                            draw_tris_fn(&mut rpass, &device, &color_bgl, color, &tris2);
                                        } else {
                                            // bevel fallback
                                            let p_outer_prev = to_ndc(px + outer_prev.0, py + outer_prev.1);
                                            let p_outer_next = to_ndc(px + outer_next.0, py + outer_next.1);
                                            let p_center = to_ndc(px, py);
                                            let tris2 = [p_outer_prev, p_center, p_outer_next];
                                            draw_tris_fn(&mut rpass, &device, &color_bgl, color, &tris2);
                                        }
                                    }
                                } else {
                                    // bevel or round (treated as bevel for now)
                                    let p_outer_prev = to_ndc(px + outer_prev.0, py + outer_prev.1);
                                    let p_outer_next = to_ndc(px + outer_next.0, py + outer_next.1);
                                    let p_center = to_ndc(px, py);
                                    let tris2 = [p_outer_prev, p_center, p_outer_next];
                                    draw_tris_fn(&mut rpass, &device, &color_bgl, color, &tris2);
                                }
                            }
                        }
                    }
                    t0 += take;
                    if use_dash {
                        rem -= take;
                        if rem <= 0.0 {
                            pat_idx = (pat_idx + 1) % pattern.len();
                            on = !on; rem = pattern[pat_idx];
                        }
                    }
                }
                prev_dir = Some(dir);
                prev_norm = Some((-dir.1, dir.0));
            }
            poly.clear();
        };

        for cmd in &path.commands {
            match *cmd {
                momentum_core::model::PathCommand::MoveTo(px, py) => {
                    // flush previous open subpath
                    flush_subpath(&mut subpath, start_pt, false);
                    subpath.clear();
                    subpath.push((tx + px, ty + py));
                    start_pt = Some((tx + px, ty + py));
                }
                momentum_core::model::PathCommand::LineTo(px, py) => {
                    subpath.push((tx + px, ty + py));
                }
                momentum_core::model::PathCommand::Close => {
                    // close current polyline
                    flush_subpath(&mut subpath, start_pt, true);
                    subpath.clear();
                    start_pt = None;
                }
                _ => { /* TODO: curve flattening not implemented yet */ }
            }
        }
        // Flush trailing open subpath
        flush_subpath(&mut subpath, start_pt, false);
        Ok(())
    }

    fn draw_text(&mut self, _transform: &Transform, _span: &TextSpan) -> Result<(), RenderError> { Ok(()) }

    fn measure_text(&mut self, _span: &TextSpan) -> Result<TextMetrics, RenderError> { Ok(TextMetrics::default()) }

    fn upload_image(&mut self, _id: ImageId, _data: &[u8]) -> Result<(), RenderError> { Ok(()) }

    fn draw_image(&mut self, _id: ImageId, _dest: Rect, _transform: &Transform, _tint: Option<momentum_core::model::Color>) -> Result<(), RenderError> { Ok(()) }

    fn draw_scale_handle(&mut self, handle: &momentum_core::model::ScaleHandle) -> Result<(), RenderError> {
        // Por ahora, implementación básica que delega a Canvas2D para los handles
        // En una implementación completa se usaría GPU shaders
        Ok(())
    }
}
