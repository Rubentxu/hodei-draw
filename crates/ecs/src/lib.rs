//! ECS standalone crate integrating bevy_ecs with momentum-core models.

use bevy_ecs::{prelude::*, schedule::Schedule};
use momentum_core::usecases::Document;
use momentum_core::model::{Style, Transform, Shape, Color, EntityId, Hitbox};
use momentum_core::ports::RenderPort;
use bevy_ecs::system::NonSendMut;

#[cfg(target_arch = "wasm32")]
use js_sys;
#[cfg(target_arch = "wasm32")]
use web_sys;

// Macro de logging condicional para WASM
#[cfg(target_arch = "wasm32")]
macro_rules! log {
    ($($arg:tt)*) => {
        web_sys::console::log_1(&format!($($arg)*).into());
    };
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! log {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

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
    pub ctrl_key: bool, // Para multi-selección
    pub shift_key: bool, // Para selección rango (futuro)
}

// Nuevos eventos para manipulación
#[derive(Debug, Clone, Copy)]
pub struct MoveStart {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct MoveUpdate {
    pub dx: f32, // Delta X desde el inicio del movimiento
    pub dy: f32, // Delta Y desde el inicio del movimiento
}

#[derive(Debug, Clone, Copy)]
pub struct MoveEnd;

// Eventos para escalado
#[derive(Debug, Clone, Copy)]
pub struct ScaleStart {
    pub handle_type: momentum_core::model::HandleType,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ScaleUpdate {
    pub dx: f32, // Delta X desde el inicio del escalado
    pub dy: f32, // Delta Y desde el inicio del escalado
}

#[derive(Debug, Clone, Copy)]
pub struct ScaleEnd;

#[derive(Resource, Default)]
pub struct InputQueue {
    pub pointer_down: Vec<PointerDown>,
    pub create_rect: Vec<CreateRect>,
    pub create_ellipse: Vec<CreateEllipse>,
    pub create_line: Vec<CreateLine>,
    pub move_start: Vec<MoveStart>,
    pub move_update: Vec<MoveUpdate>,
    pub move_end: Vec<MoveEnd>,
    pub scale_start: Vec<ScaleStart>,
    pub scale_update: Vec<ScaleUpdate>,
    pub scale_end: Vec<ScaleEnd>,
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

// Recurso para gestionar el estado de movimiento
#[derive(Resource, Default)]
pub struct MoveState {
    pub is_moving: bool,
    pub initial_positions: Vec<(EntityId, Transform)>, // Posiciones iniciales de entidades seleccionadas
}

// Recurso para gestionar el estado de escalado
#[derive(Resource, Default)]
pub struct ScaleState {
    pub is_scaling: bool,
    pub handle_type: Option<momentum_core::model::HandleType>,
    pub initial_transforms: Vec<(EntityId, Transform)>, // Transformaciones iniciales
    pub initial_bounds: momentum_core::model::BoundingBox, // Bounding box inicial del grupo
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

/// Hit test usando el nuevo sistema de hitbox con fallback al shape
fn hit_test_entity(click_x: f32, click_y: f32, entity_id: EntityId, transform: &Transform, shape: &Shape, core: &Document) -> bool {
    if let Some(hitbox) = core.get_hitbox(entity_id) {
        hitbox.hit_test(click_x, click_y, transform, shape)
    } else {
        // Fallback: usar shape como hitbox con tolerancia por defecto
        let default_hitbox = Hitbox::from_shape(shape);
        default_hitbox.hit_test(click_x, click_y, transform, shape)
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
    
    // Coleccionar eventos primero para evitar problemas de borrow checker
    let events: Vec<_> = queue.pointer_down.drain(..).collect();
    
    for event in &events {
        // Convertir coordenadas de CSS px a píxeles físicos para hit testing
        let click_x = event.x * dpr.0;
        let click_y = event.y * dpr.0;
        
        let mut handle_clicked = false;
        
        // PRIORIDAD 1: Verificar si se hizo clic en un scale handle (solo si hay selección)
        if !selection.selected.is_empty() {
            // Calcular bounding box combinado de todas las entidades seleccionadas
            let mut min_x = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            
            for selected_id in &selection.selected {
                for (id, transform, _style, shape) in &core.0.entities {
                    if id == selected_id {
                        let bbox = momentum_core::model::BoundingBox::from_shape(transform, shape);
                        min_x = min_x.min(bbox.x);
                        max_x = max_x.max(bbox.x + bbox.width);
                        min_y = min_y.min(bbox.y);
                        max_y = max_y.max(bbox.y + bbox.height);
                        break;
                    }
                }
            }
            
            if min_x < f32::INFINITY {
                // Crear bounding box combinado y convertir a píxeles físicos
                let combined_bbox = momentum_core::model::BoundingBox {
                    x: min_x * dpr.0,
                    y: min_y * dpr.0,
                    width: (max_x - min_x) * dpr.0,
                    height: (max_y - min_y) * dpr.0,
                };
                
                // Generar handles y verificar si se hizo clic en uno
                let handle_size = 10.0 * dpr.0;
                let handles = combined_bbox.generate_handles(handle_size);
                
                // Usar hitbox circular más grande para mejor interacción con handles
                let hit_radius = (handle_size * 0.75).max(12.0 * dpr.0); // Más grande que visual
                
                for handle in handles {
                    let handle_center_x = handle.x + handle.size / 2.0;
                    let handle_center_y = handle.y + handle.size / 2.0;
                    
                    let dx = click_x - handle_center_x;
                    let dy = click_y - handle_center_y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance <= hit_radius {
                        // Se hizo clic en un handle - iniciar escalado
                        log!("Clicked on scale handle: {:?} (hit radius: {})", handle.handle_type, hit_radius);
                        queue.scale_start.push(ScaleStart {
                            handle_type: handle.handle_type,
                            x: event.x,
                            y: event.y,
                        });
                        handle_clicked = true;
                        break; // Solo procesar el primer handle encontrado
                    }
                }
            }
        }
        
        // PRIORIDAD 2: Hit test de entidades (solo si no se hizo clic en un handle)
        if !handle_clicked {
            let mut found_entity = None;
            
            for (id, transform, _style, shape) in &core.0.entities {
                // Usar el nuevo sistema de hitbox con fallback al shape
                let transform_physical = Transform {
                    x: transform.x * dpr.0,
                    y: transform.y * dpr.0,
                    scale_x: transform.scale_x * dpr.0,
                    scale_y: transform.scale_y * dpr.0,
                    rotation: transform.rotation,
                };
                
                let mut shape_physical = shape.clone();
                match shape_physical {
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
                
                if hit_test_entity(click_x, click_y, *id, &transform_physical, &shape_physical, &core.0) {
                    found_entity = Some(*id);
                    break; // Seleccionar la primera entidad encontrada
                }
            }
            
            // Actualizar selección con soporte para multi-selección
            if let Some(entity_id) = found_entity {
                log!("Hit detected on entity {}", entity_id.0);
                if event.ctrl_key {
                    // Ctrl/Cmd+click: toggle la selección de la entidad
                    selection.toggle(entity_id);
                    log!("Toggled entity {} selection", entity_id.0);
                } else {
                    // Click normal: selección única (reemplaza la anterior)
                    selection.clear();
                    selection.select(entity_id);
                    log!("Selected entity {}", entity_id.0);
                }
                log!("Current selection: {:?}", selection.selected);
            } else {
                log!("No entity hit");
                // Clic en espacio vacío - limpiar selección solo si no se mantiene Ctrl
                if !event.ctrl_key {
                    selection.clear();
                    log!("Cleared selection");
                }
            }
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

fn handle_move_start_system(
    mut queue: ResMut<InputQueue>,
    mut move_state: ResMut<MoveState>,
    selection: Res<Selection>,
    core: Res<CoreDoc>,
) {
    if queue.move_start.is_empty() { return; }
    for _ev in queue.move_start.drain(..) {
        if !selection.selected.is_empty() {
            // Capturar posiciones iniciales de todas las entidades seleccionadas
            move_state.initial_positions.clear();
            for selected_id in &selection.selected {
                for (id, transform, _style, _shape) in &core.0.entities {
                    if id == selected_id {
                        move_state.initial_positions.push((*selected_id, *transform));
                        break;
                    }
                }
            }
            move_state.is_moving = true;
        }
    }
}

fn handle_move_update_system(
    mut queue: ResMut<InputQueue>,
    move_state: Res<MoveState>,
    mut core: ResMut<CoreDoc>,
    dpr: Res<CanvasDpr>,
) {
    if queue.move_update.is_empty() || !move_state.is_moving { return; }
    for ev in queue.move_update.drain(..) {
        // Convertir delta de CSS px a píxeles físicos
        let dx = ev.dx * dpr.0;
        let dy = ev.dy * dpr.0;
        
        // Actualizar posiciones de entidades seleccionadas
        for (move_id, initial_transform) in &move_state.initial_positions {
            for (id, transform, _style, _shape) in &mut core.0.entities {
                if id == move_id {
                    transform.x = initial_transform.x + dx;
                    transform.y = initial_transform.y + dy;
                    break;
                }
            }
        }
    }
}

fn handle_move_end_system(
    mut queue: ResMut<InputQueue>,
    mut move_state: ResMut<MoveState>,
) {
    if queue.move_end.is_empty() { return; }
    for _ev in queue.move_end.drain(..) {
        move_state.is_moving = false;
        move_state.initial_positions.clear();
    }
}

fn handle_scale_start_system(
    mut queue: ResMut<InputQueue>,
    mut scale_state: ResMut<ScaleState>,
    selection: Res<Selection>,
    core: Res<CoreDoc>,
) {
    if queue.scale_start.is_empty() { return; }
    for ev in queue.scale_start.drain(..) {
        if !selection.selected.is_empty() {
            // Capturar transformaciones iniciales y calcular bounding box del grupo
            scale_state.initial_transforms.clear();
            let mut min_x = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_y = f32::NEG_INFINITY;
            
            for selected_id in &selection.selected {
                for (id, transform, _style, shape) in &core.0.entities {
                    if id == selected_id {
                        scale_state.initial_transforms.push((*selected_id, *transform));
                        let bbox = momentum_core::model::BoundingBox::from_shape(transform, shape);
                        min_x = min_x.min(bbox.x);
                        max_x = max_x.max(bbox.x + bbox.width);
                        min_y = min_y.min(bbox.y);
                        max_y = max_y.max(bbox.y + bbox.height);
                        break;
                    }
                }
            }
            
            scale_state.initial_bounds = momentum_core::model::BoundingBox {
                x: min_x,
                y: min_y,
                width: max_x - min_x,
                height: max_y - min_y,
            };
            scale_state.handle_type = Some(ev.handle_type);
            scale_state.is_scaling = true;
        }
    }
}

fn handle_scale_update_system(
    mut queue: ResMut<InputQueue>,
    scale_state: Res<ScaleState>,
    mut core: ResMut<CoreDoc>,
) {
    if queue.scale_update.is_empty() || !scale_state.is_scaling { return; }
    for ev in queue.scale_update.drain(..) {
        let Some(handle_type) = scale_state.handle_type else { continue; };
        
        // Calcular factor de escala basado en el handle y el delta
        let (scale_x, scale_y) = match handle_type {
            momentum_core::model::HandleType::TopLeft => {
                let factor_x = 1.0 - ev.dx / scale_state.initial_bounds.width.max(1.0);
                let factor_y = 1.0 - ev.dy / scale_state.initial_bounds.height.max(1.0);
                (factor_x.max(0.1), factor_y.max(0.1))
            },
            momentum_core::model::HandleType::TopRight => {
                let factor_x = 1.0 + ev.dx / scale_state.initial_bounds.width.max(1.0);
                let factor_y = 1.0 - ev.dy / scale_state.initial_bounds.height.max(1.0);
                (factor_x.max(0.1), factor_y.max(0.1))
            },
            momentum_core::model::HandleType::BottomLeft => {
                let factor_x = 1.0 - ev.dx / scale_state.initial_bounds.width.max(1.0);
                let factor_y = 1.0 + ev.dy / scale_state.initial_bounds.height.max(1.0);
                (factor_x.max(0.1), factor_y.max(0.1))
            },
            momentum_core::model::HandleType::BottomRight => {
                let factor_x = 1.0 + ev.dx / scale_state.initial_bounds.width.max(1.0);
                let factor_y = 1.0 + ev.dy / scale_state.initial_bounds.height.max(1.0);
                (factor_x.max(0.1), factor_y.max(0.1))
            },
            momentum_core::model::HandleType::Top => (1.0, (1.0 - ev.dy / scale_state.initial_bounds.height.max(1.0)).max(0.1)),
            momentum_core::model::HandleType::Bottom => (1.0, (1.0 + ev.dy / scale_state.initial_bounds.height.max(1.0)).max(0.1)),
            momentum_core::model::HandleType::Left => ((1.0 - ev.dx / scale_state.initial_bounds.width.max(1.0)).max(0.1), 1.0),
            momentum_core::model::HandleType::Right => ((1.0 + ev.dx / scale_state.initial_bounds.width.max(1.0)).max(0.1), 1.0),
        };
        
        // Aplicar escala a todas las entidades seleccionadas
        for (scale_id, initial_transform) in &scale_state.initial_transforms {
            for (id, transform, _style, _shape) in &mut core.0.entities {
                if id == scale_id {
                    transform.scale_x = initial_transform.scale_x * scale_x;
                    transform.scale_y = initial_transform.scale_y * scale_y;
                    break;
                }
            }
        }
    }
}

fn handle_scale_end_system(
    mut queue: ResMut<InputQueue>,
    mut scale_state: ResMut<ScaleState>,
) {
    if queue.scale_end.is_empty() { return; }
    for _ev in queue.scale_end.drain(..) {
        scale_state.is_scaling = false;
        scale_state.handle_type = None;
        scale_state.initial_transforms.clear();
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
        world.insert_resource(MoveState::default());
        world.insert_resource(ScaleState::default());
        world.insert_resource(CanvasSize::default());
        world.insert_resource(CanvasDpr(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems((
            tick_system,
            handle_pointer_down_system,
            handle_create_rect_system,
            handle_create_ellipse_system,
            handle_create_line_system,
            handle_move_start_system,
            handle_move_update_system,
            handle_move_end_system,
            handle_scale_start_system,
            handle_scale_update_system,
            handle_scale_end_system,
            render_system_with_selection_and_handles,
        ));

        Self { world, schedule }
    }
}

pub struct PointerDownResult {
    pub clicked_handle_type: Option<u8>,
    pub entity_selected: bool,
}

impl MomentumEcsApp {
    pub fn new() -> Self { Self::default() }
    pub fn run_frame(&mut self) { self.schedule.run(&mut self.world); }
    pub fn frames(&mut self) -> u64 { self.world.resource::<AppState>().frames }
    pub fn send_pointer_down(&mut self, x: f32, y: f32) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.pointer_down.push(PointerDown { x, y, ctrl_key: false, shift_key: false });
    }
    
    pub fn send_pointer_down_with_modifiers(&mut self, x: f32, y: f32, ctrl_key: bool, shift_key: bool) -> PointerDownResult {
        // Detectar handle click inmediatamente antes de añadir a queue
        let handle_clicked = self.detect_handle_click(x, y);
        
        let mut q = self.world.resource_mut::<InputQueue>();
        q.pointer_down.push(PointerDown { x, y, ctrl_key, shift_key });
        
        PointerDownResult {
            clicked_handle_type: handle_clicked,
            entity_selected: false, // TODO: implementar detección de entidad si necesario
        }
    }
    
    pub fn detect_handle_click(&mut self, x: f32, y: f32) -> Option<u8> {
        let selection = self.world.resource::<Selection>();
        let core = self.world.resource::<CoreDoc>();
        let dpr = self.world.resource::<CanvasDpr>();
        
        if selection.selected.is_empty() {
            return None;
        }
        
        // Convertir coordenadas de CSS px a píxeles físicos
        let click_x = x * dpr.0;
        let click_y = y * dpr.0;
        
        // Calcular bounding box combinado de todas las entidades seleccionadas
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        
        for selected_id in &selection.selected {
            for (id, transform, _style, shape) in &core.0.entities {
                if id == selected_id {
                    let bbox = momentum_core::model::BoundingBox::from_shape(transform, shape);
                    min_x = min_x.min(bbox.x);
                    max_x = max_x.max(bbox.x + bbox.width);
                    min_y = min_y.min(bbox.y);
                    max_y = max_y.max(bbox.y + bbox.height);
                    break;
                }
            }
        }
        
        if min_x >= f32::INFINITY {
            return None;
        }
        
        // Crear bounding box combinado y convertir a píxeles físicos
        let combined_bbox = momentum_core::model::BoundingBox {
            x: min_x * dpr.0,
            y: min_y * dpr.0,
            width: (max_x - min_x) * dpr.0,
            height: (max_y - min_y) * dpr.0,
        };
        
        // Generar handles y verificar si se hizo clic en uno
        let handle_size = 10.0 * dpr.0;
        let handles = combined_bbox.generate_handles(handle_size);
        
        // Usar hitbox circular más grande para mejor interacción (mismo que en handle_pointer_down_system)
        let hit_radius = (handle_size * 0.75).max(12.0 * dpr.0);
        
        for handle in handles {
            let handle_center_x = handle.x + handle.size / 2.0;
            let handle_center_y = handle.y + handle.size / 2.0;
            
            let dx = click_x - handle_center_x;
            let dy = click_y - handle_center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance <= hit_radius {
                log!("Detected handle click immediately: {:?} (hit radius: {})", handle.handle_type, hit_radius);
                return Some(handle.handle_type.to_u8());
            }
        }
        
        None
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
    
    pub fn send_move_start(&mut self, x: f32, y: f32) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.move_start.push(MoveStart { x, y });
    }
    
    pub fn send_move_update(&mut self, dx: f32, dy: f32) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.move_update.push(MoveUpdate { dx, dy });
    }
    
    pub fn send_move_end(&mut self) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.move_end.push(MoveEnd);
    }
    
    pub fn send_scale_start(&mut self, handle_type: momentum_core::model::HandleType, x: f32, y: f32) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.scale_start.push(ScaleStart { handle_type, x, y });
    }
    
    pub fn send_scale_update(&mut self, dx: f32, dy: f32) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.scale_update.push(ScaleUpdate { dx, dy });
    }
    
    pub fn send_scale_end(&mut self) {
        let mut q = self.world.resource_mut::<InputQueue>();
        q.scale_end.push(ScaleEnd);
    }
    
    pub fn get_selected_entities(&self) -> Vec<EntityId> {
        self.world.resource::<Selection>().selected.clone()
    }
    
    pub fn is_moving(&self) -> bool {
        self.world.resource::<MoveState>().is_moving
    }
    
    pub fn is_scaling(&self) -> bool {
        self.world.resource::<ScaleState>().is_scaling
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

fn render_system_with_selection_and_handles(
    renderer: Option<NonSendMut<RendererBox>>, 
    size: Res<CanvasSize>,
    dpr: Res<CanvasDpr>,
    core: Res<CoreDoc>,
    selection: Res<Selection>,
    move_state: Res<MoveState>,
    _scale_state: Res<ScaleState>,
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
    
    // Dibujar scale handles para entidades seleccionadas (solo si no estamos en modo movimiento)
    if !move_state.is_moving && !selection.selected.is_empty() {
        // DEBUG: Log selection state
        log!("Selected entities: {:?}", selection.selected);
        
        // Calcular bounding box combinado de todas las entidades seleccionadas
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        
        for selected_id in &selection.selected {
            for (id, transform, _style, shape) in &core.0.entities {
                if id == selected_id {
                    let bbox = momentum_core::model::BoundingBox::from_shape(transform, shape);
                    log!("Entity {} bbox: {:?}", id.0, bbox);
                    min_x = min_x.min(bbox.x);
                    max_x = max_x.max(bbox.x + bbox.width);
                    min_y = min_y.min(bbox.y);
                    max_y = max_y.max(bbox.y + bbox.height);
                    break;
                }
            }
        }
        
        if min_x < f32::INFINITY {
            // Crear bounding box combinado y convertir a píxeles físicos
            let combined_bbox = momentum_core::model::BoundingBox {
                x: min_x * dpr.0,
                y: min_y * dpr.0,
                width: (max_x - min_x) * dpr.0,
                height: (max_y - min_y) * dpr.0,
            };
            
            log!("Combined bbox (physical): {:?}", combined_bbox);
            
            // Generar y dibujar handles
            let handle_size = 10.0 * dpr.0; // 10px escalado por DPR (más grande como Excalidraw)
            let handles = combined_bbox.generate_handles(handle_size);
            
            log!("Drawing {} handles", handles.len());
            
            for handle in handles {
                log!("Drawing handle: {:?}", handle);
                let _ = renderer.0.draw_scale_handle(&handle);
            }
        }
    }
    
    let _ = renderer.0.end_frame();
}
