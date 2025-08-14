use momentum_core::{usecases::Document, model::{Style, Transform, Shape}};

fn main() {
    println!("Hodei Momentum — app-web");

    // Smoke test de integración con core
    let mut doc = Document::new();
    let id = doc.create_shape(
        Transform { x: 10.0, y: 20.0, ..Default::default() },
        Style { stroke_width: 1.0, opacity: 1.0, ..Default::default() },
        Shape::Rect { w: 100.0, h: 80.0 },
    );

    println!("Created entity {}. Count={}", id, doc.count());
}
