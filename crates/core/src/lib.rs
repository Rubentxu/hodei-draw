//! Núcleo de dominio de Hodei Momentum (hexagonal)
//! Define modelos básicos y puertos (traits) sin dependencias de plataforma.

pub mod model {
    use std::fmt;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
    pub struct EntityId(pub u64);

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct Transform {
        pub x: f32,
        pub y: f32,
        pub rotation: f32,
        pub scale_x: f32,
        pub scale_y: f32,
    }

    impl Default for Transform {
        fn default() -> Self {
            Self {
                x: 0.0,
                y: 0.0,
                rotation: 0.0,
                scale_x: 1.0,  // ¡Esto es crítico! Debe ser 1.0, no 0.0
                scale_y: 1.0,  // ¡Esto es crítico! Debe ser 1.0, no 0.0
            }
        }
    }

    #[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct Color(pub f32, pub f32, pub f32, pub f32);

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct Style {
        pub fill: Option<Color>,
        pub stroke: Option<Color>,
        pub stroke_width: f32,
        pub opacity: f32,
        #[serde(default)]
        pub stroke_cap: StrokeCap,
        #[serde(default)]
        pub stroke_join: StrokeJoin,
        #[serde(default)]
        pub dash: Vec<f32>,
        #[serde(default)]
        pub dash_offset: f32,
    }

    #[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
    pub enum StrokeCap { #[default] Butt, Square, Round }

    #[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
    pub enum StrokeJoin { #[default] Miter, Bevel, Round }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum Shape {
        Rect { w: f32, h: f32 },
        Ellipse { rx: f32, ry: f32 },
        Line { x2: f32, y2: f32 },
        Polygon { points: Vec<(f32, f32)> },
    }

    // Tipos base adicionales (contratos de puertos)
    #[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct Vec2(pub f32, pub f32);

    #[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct Rect {
        pub x: f32,
        pub y: f32,
        pub w: f32,
        pub h: f32,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum PathCommand {
        MoveTo(f32, f32),
        LineTo(f32, f32),
        QuadTo { cx: f32, cy: f32, x: f32, y: f32 },
        CubicTo { c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32 },
        Close,
    }

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct Path {
        pub commands: Vec<PathCommand>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct TextSpan {
        pub text: String,
        pub color: Color,
        pub size: f32,
        pub font_family: Option<String>,
        pub weight: Option<u16>,
    }

    #[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
    pub struct TextMetrics {
        pub width: f32,
        pub ascent: f32,
        pub descent: f32,
        pub line_gap: f32,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
    pub struct ImageId(pub u64);

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
    pub struct BlobId(pub u64);

    // Hitbox: zona invisible para detección de eventos/colisiones, separada de la representación visual
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum Hitbox {
        /// Por defecto: usar la forma geométrica como hitbox con tolerancia para líneas/bordes finos
        FromShape { tolerance: f32 },
        /// AABB (Axis-Aligned Bounding Box) explícito - más eficiente para hit testing
        Rect { x: f32, y: f32, w: f32, h: f32 },
        /// Círculo explícito - ideal para handles y elementos pequeños
        Circle { x: f32, y: f32, radius: f32 },
        /// Polígono personalizado
        Polygon { points: Vec<(f32, f32)> },
        /// Múltiples zonas de interacción (ej: handles + shape)
        Multiple(Vec<Hitbox>),
        /// Sin hitbox - elemento no interactivo
        None,
    }

    impl Default for Hitbox {
        fn default() -> Self {
            Hitbox::FromShape { tolerance: 5.0 }
        }
    }

    impl Hitbox {
        /// Crear hitbox desde una forma geométrica con tolerancia por defecto
        pub fn from_shape(shape: &Shape) -> Self {
            match shape {
                Shape::Line { .. } => Hitbox::FromShape { tolerance: 8.0 }, // Más tolerancia para líneas
                _ => Hitbox::FromShape { tolerance: 2.0 }, // Tolerancia mínima para otras formas
            }
        }

        /// Crear hitbox circular para handles de escala (más grande que su representación visual)
        pub fn scale_handle(center_x: f32, center_y: f32, visual_size: f32) -> Self {
            let hit_radius = (visual_size * 0.75).max(12.0); // 50% más grande, mínimo 12px de radio
            Hitbox::Circle { x: center_x, y: center_y, radius: hit_radius }
        }

        /// Crear hitbox rectangular con padding para accesibilidad
        pub fn accessible_rect(x: f32, y: f32, w: f32, h: f32, min_size: f32) -> Self {
            let final_w = w.max(min_size);
            let final_h = h.max(min_size);
            let offset_x = (final_w - w) / 2.0;
            let offset_y = (final_h - h) / 2.0;
            
            Hitbox::Rect { 
                x: x - offset_x, 
                y: y - offset_y, 
                w: final_w, 
                h: final_h 
            }
        }

        /// Test de hit contra un punto, considerando transform de la entidad
        pub fn hit_test(&self, click_x: f32, click_y: f32, transform: &Transform, shape: &Shape) -> bool {
            match self {
                Hitbox::FromShape { tolerance } => {
                    Self::hit_test_shape(click_x, click_y, transform, shape, *tolerance)
                },
                Hitbox::Rect { x, y, w, h } => {
                    let world_x = transform.x + x;
                    let world_y = transform.y + y;
                    let world_w = w * transform.scale_x;
                    let world_h = h * transform.scale_y;
                    
                    click_x >= world_x && click_x <= world_x + world_w &&
                    click_y >= world_y && click_y <= world_y + world_h
                },
                Hitbox::Circle { x, y, radius } => {
                    let world_x = transform.x + x * transform.scale_x;
                    let world_y = transform.y + y * transform.scale_y;
                    let world_radius = radius * transform.scale_x.max(transform.scale_y);
                    
                    let dx = click_x - world_x;
                    let dy = click_y - world_y;
                    (dx * dx + dy * dy) <= (world_radius * world_radius)
                },
                Hitbox::Polygon { points } => {
                    if points.len() < 3 { return false; }
                    
                    // Ray casting algorithm para polígonos
                    let mut inside = false;
                    let mut j = points.len() - 1;
                    for i in 0..points.len() {
                        let xi = points[i].0 * transform.scale_x + transform.x;
                        let yi = points[i].1 * transform.scale_y + transform.y;
                        let xj = points[j].0 * transform.scale_x + transform.x;
                        let yj = points[j].1 * transform.scale_y + transform.y;
                        
                        if ((yi > click_y) != (yj > click_y)) && 
                           (click_x < (xj - xi) * (click_y - yi) / (yj - yi) + xi) {
                            inside = !inside;
                        }
                        j = i;
                    }
                    inside
                },
                Hitbox::Multiple(hitboxes) => {
                    hitboxes.iter().any(|h| h.hit_test(click_x, click_y, transform, shape))
                },
                Hitbox::None => false,
            }
        }

        /// Hit test directo contra una forma geométrica (método original)
        fn hit_test_shape(click_x: f32, click_y: f32, transform: &Transform, shape: &Shape, tolerance: f32) -> bool {
            match shape {
                Shape::Rect { w, h } => {
                    let world_w = w * transform.scale_x;
                    let world_h = h * transform.scale_y;
                    click_x >= transform.x && click_x <= transform.x + world_w &&
                    click_y >= transform.y && click_y <= transform.y + world_h
                }
                Shape::Ellipse { rx, ry } => {
                    let world_rx = rx * transform.scale_x;
                    let world_ry = ry * transform.scale_y;
                    let dx = click_x - transform.x;
                    let dy = click_y - transform.y;
                    (dx * dx) / (world_rx * world_rx) + (dy * dy) / (world_ry * world_ry) <= 1.0
                }
                Shape::Line { x2, y2 } => {
                    let world_x2 = x2 * transform.scale_x;
                    let world_y2 = y2 * transform.scale_y;
                    let line_length = (world_x2 * world_x2 + world_y2 * world_y2).sqrt();
                    if line_length == 0.0 { return false; }
                    
                    let t = ((click_x - transform.x) * world_x2 + (click_y - transform.y) * world_y2) / (line_length * line_length);
                    let t_clamped = t.clamp(0.0, 1.0);
                    let proj_x = transform.x + t_clamped * world_x2;
                    let proj_y = transform.y + t_clamped * world_y2;
                    let dist = ((click_x - proj_x) * (click_x - proj_x) + (click_y - proj_y) * (click_y - proj_y)).sqrt();
                    dist <= tolerance
                }
                Shape::Polygon { points } => {
                    if points.len() < 3 { return false; }
                    let mut inside = false;
                    let mut j = points.len() - 1;
                    for i in 0..points.len() {
                        let xi = points[i].0 * transform.scale_x + transform.x;
                        let yi = points[i].1 * transform.scale_y + transform.y;
                        let xj = points[j].0 * transform.scale_x + transform.x;
                        let yj = points[j].1 * transform.scale_y + transform.y;
                        
                        if ((yi > click_y) != (yj > click_y)) && 
                           (click_x < (xj - xi) * (click_y - yi) / (yj - yi) + xi) {
                            inside = !inside;
                        }
                        j = i;
                    }
                    inside
                }
            }
        }
    }

    // Scale handles para manipulación de formas
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum HandleType {
        // Esquinas
        TopLeft,
        TopRight,
        BottomLeft, 
        BottomRight,
        // Bordes
        Top,
        Right,
        Bottom,
        Left,
    }

    impl HandleType {
        pub fn to_u8(self) -> u8 {
            match self {
                HandleType::TopLeft => 0,
                HandleType::TopRight => 1,
                HandleType::BottomLeft => 2,
                HandleType::BottomRight => 3,
                HandleType::Top => 4,
                HandleType::Right => 5,
                HandleType::Bottom => 6,
                HandleType::Left => 7,
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct ScaleHandle {
        pub handle_type: HandleType,
        pub x: f32,
        pub y: f32,
        pub size: f32,
    }

    // Información de bounding box de una forma
    #[derive(Clone, Copy, Debug, Default)]
    pub struct BoundingBox {
        pub x: f32,
        pub y: f32,
        pub width: f32,
        pub height: f32,
    }

    // Persistencia
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
    pub struct ProjectId(pub u64);

    #[derive(Clone, Copy, Debug, Serialize, Deserialize)]
    pub struct Timestamp(pub u64); // epoch millis

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ProjectMeta {
        pub id: ProjectId,
        pub name: String,
        pub updated_at: Timestamp,
        pub size: u64,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct Project {
        pub id: ProjectId,
        pub name: String,
        pub document: crate::usecases::Document,
        pub schema_version: u32,
        pub thumbnails: Vec<BlobId>,
        pub updated_at: Timestamp,
    }

    

    impl fmt::Display for EntityId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
    }

    // Utilidades para calcular bounding boxes
    impl BoundingBox {
        pub fn from_shape(transform: &Transform, shape: &Shape) -> Self {
            match shape {
                Shape::Rect { w, h } => BoundingBox {
                    x: transform.x,
                    y: transform.y,
                    width: *w * transform.scale_x,
                    height: *h * transform.scale_y,
                },
                Shape::Ellipse { rx, ry } => BoundingBox {
                    x: transform.x - rx * transform.scale_x,
                    y: transform.y - ry * transform.scale_y,
                    width: 2.0 * rx * transform.scale_x,
                    height: 2.0 * ry * transform.scale_y,
                },
                Shape::Line { x2, y2 } => {
                    let end_x = transform.x + x2 * transform.scale_x;
                    let end_y = transform.y + y2 * transform.scale_y;
                    BoundingBox {
                        x: transform.x.min(end_x),
                        y: transform.y.min(end_y),
                        width: (end_x - transform.x).abs(),
                        height: (end_y - transform.y).abs(),
                    }
                },
                Shape::Polygon { points } => {
                    if points.is_empty() {
                        return BoundingBox::default();
                    }
                    let mut min_x = f32::INFINITY;
                    let mut max_x = f32::NEG_INFINITY;
                    let mut min_y = f32::INFINITY;
                    let mut max_y = f32::NEG_INFINITY;
                    
                    for (px, py) in points {
                        let world_x = transform.x + px * transform.scale_x;
                        let world_y = transform.y + py * transform.scale_y;
                        min_x = min_x.min(world_x);
                        max_x = max_x.max(world_x);
                        min_y = min_y.min(world_y);
                        max_y = max_y.max(world_y);
                    }
                    
                    BoundingBox {
                        x: min_x,
                        y: min_y,
                        width: max_x - min_x,
                        height: max_y - min_y,
                    }
                }
            }
        }

        pub fn generate_handles(&self, handle_size: f32) -> Vec<ScaleHandle> {
            let half_size = handle_size / 2.0;
            vec![
                // Esquinas
                ScaleHandle { handle_type: HandleType::TopLeft, x: self.x - half_size, y: self.y - half_size, size: handle_size },
                ScaleHandle { handle_type: HandleType::TopRight, x: self.x + self.width - half_size, y: self.y - half_size, size: handle_size },
                ScaleHandle { handle_type: HandleType::BottomLeft, x: self.x - half_size, y: self.y + self.height - half_size, size: handle_size },
                ScaleHandle { handle_type: HandleType::BottomRight, x: self.x + self.width - half_size, y: self.y + self.height - half_size, size: handle_size },
                // Bordes
                ScaleHandle { handle_type: HandleType::Top, x: self.x + self.width / 2.0 - half_size, y: self.y - half_size, size: handle_size },
                ScaleHandle { handle_type: HandleType::Right, x: self.x + self.width - half_size, y: self.y + self.height / 2.0 - half_size, size: handle_size },
                ScaleHandle { handle_type: HandleType::Bottom, x: self.x + self.width / 2.0 - half_size, y: self.y + self.height - half_size, size: handle_size },
                ScaleHandle { handle_type: HandleType::Left, x: self.x - half_size, y: self.y + self.height / 2.0 - half_size, size: handle_size },
            ]
        }
    }
}

pub mod ports {
    use super::model::{BlobId, Path, Project, ProjectId, ProjectMeta, Rect, Shape, TextMetrics, TextSpan, Transform, ImageId, ScaleHandle};
    use thiserror::Error;

    #[derive(Debug, Error)]
    pub enum RenderError {
        #[error("Renderer initialization failed")] Initialization,
        #[error("Device lost")] DeviceLost,
        #[error("Surface lost")] SurfaceLost,
        #[error("Out of memory")] OutOfMemory,
        #[error("Invalid input")] InvalidInput,
        #[error("Unsupported operation")] Unsupported,
        #[error("Text shaping error")] TextShaping,
        #[error("Upload failed")] UploadFailed,
        #[error("Other: {0}")] Other(String),
    }

    pub trait RenderPort {
        fn begin_frame(&mut self, width: u32, height: u32) -> Result<(), RenderError>;
        fn end_frame(&mut self) -> Result<(), RenderError>;
        fn set_camera(&mut self, transform_2d: [f32; 6]) -> Result<(), RenderError>; // 2D affine

        fn draw_shape(
            &mut self,
            transform: &Transform,
            shape: &Shape,
            style: &crate::model::Style,
        ) -> Result<(), RenderError>;

        fn draw_path(
            &mut self,
            transform: &Transform,
            path: &Path,
            style: &crate::model::Style,
        ) -> Result<(), RenderError>;

        fn draw_text(
            &mut self,
            transform: &Transform,
            span: &TextSpan,
        ) -> Result<(), RenderError>;

        fn measure_text(&mut self, span: &TextSpan) -> Result<TextMetrics, RenderError>;

        fn upload_image(&mut self, id: ImageId, data: &[u8]) -> Result<(), RenderError>;
        fn draw_image(
            &mut self,
            id: ImageId,
            dest: Rect,
            transform: &Transform,
            tint: Option<crate::model::Color>,
        ) -> Result<(), RenderError>;

        fn draw_scale_handle(
            &mut self,
            handle: &ScaleHandle,
        ) -> Result<(), RenderError>;
    }

    #[derive(Debug, thiserror::Error)]
    pub enum StorageError {
        #[error("Not found")] NotFound,
        #[error("Conflict")] Conflict,
        #[error("Quota exceeded")] QuotaExceeded,
        #[error("Serialization error")] Serialization,
        #[error("Deserialization error")] Deserialization,
        #[error("Backend: {0}")] Backend(String),
        #[error("Unsupported")] Unsupported,
        #[error("Other: {0}")] Other(String),
    }

    pub trait StoragePort {
        fn save_project(&mut self, project: Project) -> Result<ProjectId, StorageError>;
        fn load_project(&mut self, id: ProjectId) -> Result<Project, StorageError>;
        fn list_projects(&mut self) -> Result<Vec<ProjectMeta>, StorageError>;
        fn delete_project(&mut self, id: ProjectId) -> Result<(), StorageError>;
        fn put_blob(&mut self, id: BlobId, bytes: &[u8]) -> Result<(), StorageError>;
        fn get_blob(&mut self, id: BlobId) -> Result<Option<Vec<u8>>, StorageError>;
        fn migrate(&mut self, from: u32, to: u32) -> Result<(), StorageError>;
    }
}

pub mod usecases {
    use super::model::{EntityId, Shape, Style, Transform, Hitbox};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct Document {
        pub entities: Vec<(EntityId, Transform, Style, Shape)>,
        /// Hitboxes opcionales para cada entidad - None significa usar shape como hitbox
        #[serde(default)]
        pub hitboxes: Vec<(EntityId, Hitbox)>,
        next_id: u64,
    }

    impl Document {
        pub fn new() -> Self { 
            Self { 
                entities: Vec::new(), 
                hitboxes: Vec::new(),
                next_id: 1 
            } 
        }
        
        pub fn create_shape(&mut self, transform: Transform, style: Style, shape: Shape) -> EntityId {
            let id = EntityId(self.next_id);
            self.next_id += 1;
            self.entities.push((id, transform, style, shape));
            id
        }
        
        pub fn create_shape_with_hitbox(&mut self, transform: Transform, style: Style, shape: Shape, hitbox: Hitbox) -> EntityId {
            let id = self.create_shape(transform, style, shape);
            self.hitboxes.push((id, hitbox));
            id
        }
        
        pub fn set_hitbox(&mut self, entity_id: EntityId, hitbox: Hitbox) {
            // Remover hitbox existente si lo hay
            self.hitboxes.retain(|(id, _)| *id != entity_id);
            self.hitboxes.push((entity_id, hitbox));
        }
        
        pub fn get_hitbox(&self, entity_id: EntityId) -> Option<&Hitbox> {
            self.hitboxes.iter()
                .find(|(id, _)| *id == entity_id)
                .map(|(_, hitbox)| hitbox)
        }
        
        pub fn remove_hitbox(&mut self, entity_id: EntityId) {
            self.hitboxes.retain(|(id, _)| *id != entity_id);
        }
        
        pub fn count(&self) -> usize { self.entities.len() }
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
