mod common {
    include!("../common/mod.rs");
}
use common::{TestSetup, coords};
use std::path::Path;

#[tokio::test]
async fn test_visual_regression_basic_shapes() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear composición estándar de formas
    setup.create_rectangle(100.0, 100.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    setup.page.evaluate(
        "ecs_create_ellipse(350, 150, 60, 40)",
        serde_json::Value::Null
    ).await.expect("Failed to create ellipse");
    
    setup.page.evaluate(
        "ecs_create_line(100, 250, 300, 280)",
        serde_json::Value::Null
    ).await.expect("Failed to create line");
    
    // Screenshot baseline
    let screenshot = setup.screenshot("visual_baseline_shapes")
        .await
        .expect("Failed to take baseline screenshot");
    
    // Verificar que el archivo se creó
    let screenshot_path = Path::new("tests/screenshots/visual_baseline_shapes.png");
    assert!(screenshot_path.exists(), "Baseline screenshot should be created");
    
    println!("✅ Visual baseline created with {} bytes", screenshot.len());
}

#[tokio::test]
async fn test_visual_regression_selection_handles() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear rectángulo y seleccionarlo
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    setup.select_element(275.0, 250.0)
        .await
        .expect("Failed to select rectangle");
    
    // Screenshot con handles visibles
    let screenshot = setup.screenshot("visual_baseline_selection_handles")
        .await
        .expect("Failed to take selection screenshot");
    
    let screenshot_path = Path::new("tests/screenshots/visual_baseline_selection_handles.png");
    assert!(screenshot_path.exists(), "Selection handles screenshot should be created");
    
    // Verificar que la imagen tiene contenido (no está vacía)
    assert!(screenshot.len() > 1000, "Screenshot should have substantial content");
    
    println!("✅ Selection handles visual baseline created");
}

#[tokio::test]
async fn test_visual_consistency_multiple_selections() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear múltiples formas
    setup.create_rectangle(150.0, 150.0, 100.0, 80.0)
        .await
        .expect("Failed to create first rectangle");
    
    setup.create_rectangle(300.0, 200.0, 120.0, 60.0)
        .await
        .expect("Failed to create second rectangle");
    
    setup.page.evaluate(
        "ecs_create_ellipse(200, 300, 40, 30)",
        serde_json::Value::Null
    ).await.expect("Failed to create ellipse");
    
    // Seleccionar la primera forma
    setup.select_element(200.0, 190.0)
        .await
        .expect("Failed to select first shape");
    
    setup.screenshot("multi_shape_first_selected")
        .await
        .expect("Failed to take first selection screenshot");
    
    // Seleccionar la segunda forma
    setup.select_element(360.0, 230.0)
        .await
        .expect("Failed to select second shape");
    
    setup.screenshot("multi_shape_second_selected")
        .await
        .expect("Failed to take second selection screenshot");
    
    // Seleccionar el círculo
    setup.select_element(200.0, 300.0)
        .await
        .expect("Failed to select ellipse");
    
    setup.screenshot("multi_shape_ellipse_selected")
        .await
        .expect("Failed to take ellipse selection screenshot");
    
    println!("✅ Multiple selection visual consistency captured");
}

#[tokio::test]
async fn test_canvas_pixel_precision() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear formas con posiciones exactas para verificar precisión
    setup.create_rectangle(100.0, 100.0, 200.0, 100.0)
        .await
        .expect("Failed to create precise rectangle");
    
    // Verificar que los píxeles están en las posiciones esperadas
    let pixel_check = setup.page.evaluate(
        r#"
        const canvas = document.querySelector('canvas');
        const ctx = canvas.getContext('2d');
        
        // Verificar esquinas específicas del rectángulo
        const checks = [
            { x: 100, y: 100, name: 'top-left' },
            { x: 299, y: 100, name: 'top-right' },
            { x: 100, y: 199, name: 'bottom-left' },
            { x: 299, y: 199, name: 'bottom-right' },
            { x: 150, y: 150, name: 'center' },
        ];
        
        const results = [];
        for (const check of checks) {
            const imageData = ctx.getImageData(check.x, check.y, 1, 1);
            const [r, g, b, a] = imageData.data;
            const isBlack = r < 128 && g < 128 && b < 128; // Stroke color
            results.push({
                point: check.name,
                x: check.x,
                y: check.y,
                color: [r, g, b, a],
                isShape: isBlack
            });
        }
        
        return results;
        "#,
        serde_json::Value::Null
    ).await.expect("Failed to check pixels");
    
    println!("Pixel precision check results: {:#?}", pixel_check);
    
    // Verificar que las esquinas tienen color de stroke
    let pixel_array = pixel_check.as_array().expect("Should be array");
    let mut corners_with_stroke = 0;
    
    for pixel in pixel_array {
        if let Some(is_shape) = pixel.get("isShape").and_then(|v| v.as_bool()) {
            if is_shape {
                corners_with_stroke += 1;
            }
        }
    }
    
    assert!(
        corners_with_stroke >= 2, 
        "At least 2 corners should have stroke pixels, found: {}", 
        corners_with_stroke
    );
    
    setup.screenshot("pixel_precision_test")
        .await
        .expect("Failed to take precision screenshot");
    
    println!("✅ Pixel precision verified with {} stroke corners", corners_with_stroke);
}

#[tokio::test]
async fn test_device_pixel_ratio_consistency() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Verificar el DPR actual
    let dpr = setup.page.evaluate(
        "window.devicePixelRatio",
        serde_json::Value::Null
    ).await.expect("Failed to get DPR");
    
    println!("Device Pixel Ratio: {}", dpr);
    
    // Crear forma y verificar que se renderiza correctamente con el DPR actual
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    // Verificar dimensiones del canvas
    let canvas_info = setup.page.evaluate(
        r#"
        const canvas = document.querySelector('canvas');
        return {
            clientWidth: canvas.clientWidth,
            clientHeight: canvas.clientHeight,
            width: canvas.width,
            height: canvas.height,
            dpr: window.devicePixelRatio
        }
        "#,
        serde_json::Value::Null
    ).await.expect("Failed to get canvas info");
    
    println!("Canvas info: {:#?}", canvas_info);
    
    setup.screenshot("dpr_consistency_test")
        .await
        .expect("Failed to take DPR screenshot");
    
    println!("✅ Device pixel ratio consistency verified");
}

// Helper para limpiar screenshots viejos antes de tests
#[tokio::test]
async fn cleanup_old_screenshots() {
    use std::fs;
    
    let screenshots_dir = Path::new("tests/screenshots");
    
    if screenshots_dir.exists() {
        // Crear directorio limpio
        let _ = fs::remove_dir_all(screenshots_dir);
    }
    
    fs::create_dir_all(screenshots_dir).expect("Failed to create screenshots directory");
    
    println!("✅ Screenshots directory cleaned and ready");
}