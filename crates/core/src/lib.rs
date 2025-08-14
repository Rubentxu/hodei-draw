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
}

pub mod ports {
    use super::model::{BlobId, Path, Project, ProjectId, ProjectMeta, Rect, Shape, TextMetrics, TextSpan, Transform, ImageId};
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
    use super::model::{EntityId, Shape, Style, Transform};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Default, Serialize, Deserialize)]
    pub struct Document {
        pub entities: Vec<(EntityId, Transform, Style, Shape)>,
        next_id: u64,
    }

    impl Document {
        pub fn new() -> Self { Self { entities: Vec::new(), next_id: 1 } }
        pub fn create_shape(&mut self, transform: Transform, style: Style, shape: Shape) -> EntityId {
            let id = EntityId(self.next_id);
            self.next_id += 1;
            self.entities.push((id, transform, style, shape));
            id
        }
        pub fn count(&self) -> usize { self.entities.len() }
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
