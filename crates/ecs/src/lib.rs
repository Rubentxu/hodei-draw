//! ECS standalone crate integrating bevy_ecs with momentum-core models.

use bevy_ecs::{prelude::*, schedule::Schedule};
use momentum_core::usecases::Document;
use momentum_core::model::{Style, Transform, Shape, Color};
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
}

fn handle_pointer_down_system(
    mut queue: ResMut<InputQueue>,
    _core: ResMut<CoreDoc>,
) {
    // Ya no creamos un rectángulo automático en pointer_down para evitar duplicados
    // cuando existe un flujo de creación por arrastre. Simplemente vaciamos la cola
    // por ahora (se puede usar para selección u otras interacciones en el futuro).
    if queue.pointer_down.is_empty() { return; }
    queue.pointer_down.clear();
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
        world.insert_resource(CanvasSize::default());
        world.insert_resource(CanvasDpr(1.0));

        let mut schedule = Schedule::default();
        schedule.add_systems((
            tick_system,
            handle_pointer_down_system,
            handle_create_rect_system,
            render_system,
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

fn render_system(
    renderer: Option<NonSendMut<RendererBox>>, 
    size: Res<CanvasSize>,
    dpr: Res<CanvasDpr>,
    core: Res<CoreDoc>,
) {
    // Si no hay renderer (por ejemplo, WebGPU no disponible), omitir el render sin hacer panic.
    let Some(mut renderer) = renderer else { return; };
    // Comenzar frame con tamaño actual
    let _ = renderer.0.begin_frame(size.w, size.h);
    // Dibujar cada entidad del documento
    for (_id, transform, style, shape) in &core.0.entities {
        // Escalar coordenadas (CSS px) a píxeles físicos usando DPR
        let mut t = *transform;
        t.x *= dpr.0;
        t.y *= dpr.0;
        let mut s_shape = shape.clone();
        if let Shape::Rect { w, h } = s_shape {
            s_shape = Shape::Rect { w: w * dpr.0, h: h * dpr.0 };
        }
        let mut s = style.clone();
        s.stroke_width *= dpr.0;
        // Escalar patrón de guiones por DPR
        if !s.dash.is_empty() {
            for d in &mut s.dash { *d *= dpr.0; }
            s.dash_offset *= dpr.0;
        }
        let _ = renderer.0.draw_shape(&t, &s_shape, &s);
    }
    let _ = renderer.0.end_frame();
}
