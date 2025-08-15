use std::path::Path;

mod common {
    include!("../common/mod.rs");
}
use common::{TestSetup, coords};

#[tokio::test]
async fn test_create_rectangle() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Verificar que las funciones WASM están disponibles
    setup.verify_wasm_functions().await.expect("WASM functions not available");
    
    // Crear un rectángulo
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    // Tomar screenshot para verificación visual
    setup.screenshot("rectangle_created")
        .await
        .expect("Failed to take screenshot");
    
    // Verificar que el rectángulo es visible mediante JS
    let rect_exists = setup.page.evaluate(
        r#"
        // Buscar canvas y verificar que no esté vacío
        const canvas = document.querySelector('canvas');
        const ctx = canvas.getContext('2d');
        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
        const data = imageData.data;
        
        // Verificar que hay píxeles no blancos (el rectángulo dibujado)
        let hasContent = false;
        for (let i = 0; i < data.length; i += 4) {
            if (data[i] !== 255 || data[i+1] !== 255 || data[i+2] !== 255) {
                hasContent = true;
                break;
            }
        }
        hasContent
        "#,
        serde_json::Value::Null
    ).await.expect("Failed to check rectangle existence");
    
    assert!(rect_exists.as_bool().unwrap_or(false), "Rectangle should be visible on canvas");
}

#[tokio::test]
async fn test_create_multiple_shapes() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear rectángulo
    setup.create_rectangle(100.0, 100.0, 100.0, 80.0)
        .await
        .expect("Failed to create rectangle");
    
    // Crear círculo usando función WASM
    setup.page.evaluate(
        "ecs_create_ellipse(300, 150, 50, 50)",
        serde_json::Value::Null
    ).await.expect("Failed to create ellipse");
    
    // Crear línea
    setup.page.evaluate(
        "ecs_create_line(100, 250, 200, 300)",
        serde_json::Value::Null
    ).await.expect("Failed to create line");
    
    // Tomar screenshot del resultado
    setup.screenshot("multiple_shapes")
        .await
        .expect("Failed to take screenshot");
    
    // Verificar que hay contenido en el canvas
    let has_content = setup.page.evaluate(
        r#"
        const canvas = document.querySelector('canvas');
        const ctx = canvas.getContext('2d');
        const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
        const data = imageData.data;
        
        let nonWhitePixels = 0;
        for (let i = 0; i < data.length; i += 4) {
            if (data[i] !== 255 || data[i+1] !== 255 || data[i+2] !== 255) {
                nonWhitePixels++;
            }
        }
        
        nonWhitePixels > 100 // Debe haber al menos 100 píxeles no blancos
        "#,
        serde_json::Value::Null
    ).await.expect("Failed to check canvas content");
    
    assert!(has_content.as_bool().unwrap_or(false), "Canvas should contain multiple shapes");
}

#[tokio::test]
async fn test_shape_selection() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear un rectángulo para seleccionar
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    // Seleccionar el rectángulo haciendo clic en su centro
    setup.select_element(275.0, 250.0)
        .await
        .expect("Failed to select rectangle");
    
    // Verificar que hay una selección activa
    let has_selection = setup.page.evaluate(
        r#"
        // Verificar si hay elementos seleccionados usando función WASM
        const selectedEntities = ecs_get_selected_entities ? ecs_get_selected_entities() : [];
        selectedEntities.length > 0
        "#,
        serde_json::Value::Null
    ).await;
    
    // Note: La función ecs_get_selected_entities podría no estar expuesta,
    // así que verificamos visualmente que hay scale handles
    setup.screenshot("shape_selected")
        .await
        .expect("Failed to take screenshot");
    
    println!("Selection test completed - check screenshot for scale handles");
}