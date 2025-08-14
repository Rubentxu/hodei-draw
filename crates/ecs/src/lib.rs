//! ECS standalone crate integrating bevy_ecs with momentum-core models.

use bevy_ecs::{prelude::*, schedule::Schedule};
use momentum_core::usecases::Document;
use momentum_core::model::{Style, Transform, Shape, Color, EntityId};
use momentum_core::ports::RenderPort;
use bevy_ecs::system::NonSendMut;

#[derive(Resource, Default)]
pub struct AppState {
    pub frames: u64,
}

#[derive(Resource, Default)]
pub struct CoreDoc(pub Document);

fn tick_system(mut state: ResMut<AppState>) {
    state.frames += 1;
}

// Input básico
#[derive(Debug, Clone, Copy)]
pub struct PointerDown {
    pub x: f32,
    pub y: f32,
}

#[derive(Resource, Default)]
pub struct InputQueue {
    pub pointer_down: Vec<PointerDown>,
    pub create_rect: Vec<CreateRect>,
    pub create_ellipse: Vec<CreateEllipse>,
    pub create_line: Vec<CreateLine>,
}

// Nuevos eventos para crear otras formas
#[derive(Debug, Clone, Copy)]
pub struct CreateEllipse { pub x: f32, pub y: f32, pub rx: f32, pub ry: f32 }

#[derive(Debug, Clone, Copy)]
pub struct CreateLine { pub x1: f32, pub y1: f32, pub x2: f32, pub y2: f32 }

// Recurso para gestionar selección
#[derive(Resource, Default)]
pub struct Selection {
    pub selected: Vec<EntityId>,
}

impl Selection {
    pub fn is_selected(&self, id: EntityId) -> bool {
        self.selected.contains(&id)
    }
    
    pub fn select(&mut self, id: EntityId) {
        if !self.is_selected(id) {
            self.selected.push(id);
        }
    }
    
    pub fn deselect(&mut self, id: EntityId) {
        self.selected.retain(|&x| x != id);
    }
    
    pub fn clear(&mut self) {
        self.selected.clear();
    }
    
    pub fn toggle(&mut self, id: EntityId) {
        if self.is_selected(id) {
            self.deselect(id);
        } else {
            self.select(id);
        }
    }
}

fn handle_pointer_down_system(
    mut queue: ResMut<InputQueue>,
    core: Res<CoreDoc>,
    mut selection: ResMut<Selection>,
    _canvas_size: Res<CanvasSize>,
    dpr: Res<CanvasDpr>,
) {
    if queue.pointer_down.is_empty() { return; }
    
    for event in queue.pointer_down.drain(..) {
        // Convertir coordenadas de CSS px a píxeles físicos para hit testing
        let click_x = event.x * dpr.0;
        let click_y = event.y * dpr.0;
        
        // Buscar entidad en la posición del clic
        let mut found_entity = None;
        
        for (id, transform, _style, shape) in &core.0.entities {
            // Convertir transform a píxeles físicos
            let entity_x = transform.x * dpr.0;
            let entity_y = transform.y * dpr.0;
            
            // Hit test basado en el tipo de forma
            let hit = match shape {
                Shape::Rect { w, h } => {
                    let w_scaled = w * dpr.0;
                    let h_scaled = h * dpr.0;
                    click_x >= entity_x && click_x <= entity_x + w_scaled &&
                    click_y >= entity_y && click_y <= entity_y + h_scaled
                }
                Shape::Ellipse { rx, ry } => {
                    let rx_scaled = rx * dpr.0;
                    let ry_scaled = ry * dpr.0;
                    let dx = click_x - entity_x;
                    let dy = click_y - entity_y;
                    (dx * dx) / (rx_scaled * rx_scaled) + (dy * dy) / (ry_scaled * ry_scaled) <= 1.0
                }
                Shape::Line { x2, y2 } => {
                    let x2_scaled = x2 * dpr.0;
                    let y2_scaled = y2 * dpr.0;
                    // Distancia punto-línea, con tolerancia
                    let tolerance = 5.0; // píxeles de tolerancia
                    let line_length = ((x2_scaled * x2_scaled) + (y2_scaled * y2_scaled)).sqrt();
                    if line_length == 0.0 { false } else {
                        let t = ((click_x - entity_x) * x2_scaled + (click_y - entity_y) * y2_scaled) / (line_length * line_length);
                        let t_clamped = t.clamp(0.0, 1.0);
                        let proj_x = entity_x + t_clamped * x2_scaled;
                        let proj_y = entity_y + t_clamped * y2_scaled;
                        let dist = ((click_x - proj_x) * (click_x - proj_x) + (click_y - proj_y) * (click_y - proj_y)).sqrt();
                        dist <= tolerance
                    }
                }
                Shape::Polygon { points } => {
                    // Hit test para polígonos usando ray casting algorithm
                    if points.len() < 3 { false } else {
                        let mut inside = false;
                        let mut j = points.len() - 1;
                        for i in 0..points.len() {
                            let xi = points[i].0 * dpr.0 + entity_x;
                            let yi = points[i].1 * dpr.0 + entity_y;
                            let xj = points[j].0 * dpr.0 + entity_x;
                            let yj = points[j].1 * dpr.0 + entity_y;
                            
                            if ((yi > click_y) != (yj > click_y)) && 
                               (click_x < (xj - xi) * (click_y - yi) / (yj - yi) + xi) {
                                inside = !inside;
                            }
                            j = i;
                        }
                        inside
                    }
                }
            };
            
            if hit {
                found_entity = Some(*id);
                break; // Seleccionar la primera entidad encontrada
            }
        }
        
        // Actualizar selección
        if let Some(entity_id) = found_entity {
            // Por ahora, una selección simple (reemplaza la selección anterior)
            // TODO: Agregar soporte para multi-selección con Ctrl/Cmd
            selection.clear();
            selection.select(entity_id);
        } else {
            // Clic en espacio vacío - limpiar selección
            selection.clear();
        }
    }
}

// Evento explícito para crear un rectángulo con dimensiones
#[derive(Debug, Clone, Copy)]
pub struct CreateRect { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }

fn handle_create_rect_system(
    mut queue: ResMut<InputQueue>,
    mut core: ResMut<CoreDoc>,
) {
    if queue.create_rect.is_empty() { return; }
    for ev in queue.create_rect.drain(..) {
        let _id = core.0.create_shape(
            Transform { x: ev.x, y: ev.y, ..Default::default() },
            Style { stroke: Some(Color(0.10, 0.12, 0.16, 1.0)), stroke_width: 2.0, opacity: 1.0, ..Default::default() },
            Shape::Rect { w: ev.w, h: ev.h },
        );
    }
}

fn handle_create_ellipse_system(
    mut queue: ResMut<InputQueue>,
    mut core: ResMut<CoreDoc>,
) {
    if queue.create_ellipse.is_empty() { return; }
    for ev in queue.create_ellipse.drain(..) {
        let _id = core.0.create_shape(
            Transform { x: ev.x, y: ev.y, ..Default::default() },
            Style { stroke: Some(Color(0.10, 0.12, 0.16, 1.0)), stroke_width: 2.0, opacity: 1.0, ..Default::default() },
            Shape::Ellipse { rx: ev.rx, ry: ev.ry },
        );
    }
}

fn handle_create_line_system(
    mut queue: ResMut<InputQueue>,
    mut core: ResMut<CoreDoc>,
) {
    if queue.create_line.is_empty() { return; }
    for ev in queue.create_line.drain(..) {
        let _id = core.0.create_shape(
            Transform { x: ev.x1, y: ev.y1, ..Default::default() },
            Style { stroke: Some(Color(0.10, 0.12, 0.16, 1.0)), stroke_width: 2.0, opacity: 1.0, ..Default::default() },
            Shape::Line { x2: ev.x2 - ev.x1, y2: ev.y2 - ev.y1 },
        );
    }
}

pub struct MomentumEcsApp {
    world: World,
    schedule: Schedule,
}

impl Default for MomentumEcsApp {
    fn default() -> Self {
        let mut world = World::new();
        world.insert_resource(AppState::default());
        world.insert_resource(CoreDoc(Document::new()));
        world.insert_resource(InputQueue::default());
        world.insert_resource(Selection::default());
        world.insert_resource(CanvasSize::default());
        world.insert_resource(CanvasDpr(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems((
            tick_system,
            handle_pointer_down_system,
            handle_create_rect_system,
            handle_create_ellipse_system,
            handle_create_line_system,
            render_system_with_selection,
        ));

        Self { world, schedule }
    }
}

impl MomentumEcsApp {
    pub fn new() -> Self { Self::default() }
    pub fn run_frame(&mut self) { self.schedule.run(&mut self.world); }
    pub fn frames(&mut self) -> u64 { self.world.resource::<AppState>().frames }
    pub fn send_pointer_down(&mut self, x: f32, y: f32) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.pointer_down.push(PointerDown { x, y });
    }
    pub fn send_create_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.create_rect.push(CreateRect { x, y, w, h });
    }
    
    pub fn send_create_ellipse(&mut self, x: f32, y: f32, rx: f32, ry: f32) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.create_ellipse.push(CreateEllipse { x, y, rx, ry });
    }
    
    pub fn send_create_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.create_line.push(CreateLine { x1, y1, x2, y2 });
    }
    
    pub fn get_selected_entities(&self) -> Vec<EntityId> {
        self.world.resource::<Selection>().selected.clone()
    }
    pub fn document(&self) -> &Document { &self.world.resource::<CoreDoc>().0 }
    pub fn set_renderer(&mut self, renderer: Box<dyn RenderPort>) {
        // Guardar como recurso NonSend, ya que el renderer no es Send/Sync en WASM
        self.world.insert_non_send_resource(RendererBox(renderer));
    }
    pub fn set_canvas_size(&mut self, w: u32, h: u32) {
        let mut sz = self.world.resource_mut::<CanvasSize>();
        sz.w = w.max(1);
        sz.h = h.max(1);
    }
    pub fn set_canvas_dpr(&mut self, dpr: f32) {
        *self.world.resource_mut::<CanvasDpr>() = CanvasDpr(dpr.max(0.5));
    }
}

// ================= Render integration =================
#[derive(Resource, Default, Clone, Copy)]
pub struct CanvasSize { pub w: u32, pub h: u32 }

#[derive(Resource, Clone, Copy)]
pub struct CanvasDpr(pub f32);

/// Wrapper para almacenar un trait object no-Send en el mundo ECS
pub struct RendererBox(pub Box<dyn RenderPort>);

fn render_system_with_selection(
    renderer: Option<NonSendMut<RendererBox>>, 
    size: Res<CanvasSize>,
    dpr: Res<CanvasDpr>,
    core: Res<CoreDoc>,
    selection: Res<Selection>,
) {
    // Si no hay renderer (por ejemplo, WebGPU no disponible), omitir el render sin hacer panic.
    let Some(mut renderer) = renderer else { return; };
    // Comenzar frame con tamaño actual
    let _ = renderer.0.begin_frame(size.w, size.h);
    
    // Dibujar cada entidad del documento
    for (id, transform, style, shape) in &core.0.entities {
        // Escalar coordenadas (CSS px) a píxeles físicos usando DPR
        let mut t = *transform;
        t.x *= dpr.0;
        t.y *= dpr.0;
        
        let mut s_shape = shape.clone();
        match s_shape {
            Shape::Rect { ref mut w, ref mut h } => {
                *w *= dpr.0;
                *h *= dpr.0;
            }
            Shape::Ellipse { ref mut rx, ref mut ry } => {
                *rx *= dpr.0;
                *ry *= dpr.0;
            }
            Shape::Line { ref mut x2, ref mut y2 } => {
                *x2 *= dpr.0;
                *y2 *= dpr.0;
            }
            Shape::Polygon { ref mut points } => {
                for (x, y) in points {
                    *x *= dpr.0;
                    *y *= dpr.0;
                }
            }
        }
        
        let mut s = style.clone();
        s.stroke_width *= dpr.0;
        
        // Escalar patrón de guiones por DPR
        if !s.dash.is_empty() {
            for d in &mut s.dash { *d *= dpr.0; }
            s.dash_offset *= dpr.0;
        }
        
        // Modificar estilo si la entidad está seleccionada
        if selection.is_selected(*id) {
            // Hacer el stroke más grueso y cambiar el color para indicar selección
            s.stroke_width = (s.stroke_width).max(3.0 * dpr.0);
            s.stroke = Some(Color(0.0, 0.4, 0.8, 1.0)); // Azul para selección
        }
        
        let _ = renderer.0.draw_shape(&t, &s_shape, &s);
    }
    
    let _ = renderer.0.end_frame();
}
