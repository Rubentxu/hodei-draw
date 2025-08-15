mod common {
    include!("../common/mod.rs");
}
use common::{TestSetup, cursor, coords};

#[tokio::test]
async fn test_cursor_changes_on_shape_hover() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear un rectángulo
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    // Test 1: Cursor en área vacía debe ser 'auto'
    setup.hover_at(100.0, 100.0).await.expect("Failed to hover empty area");
    let cursor = setup.get_cursor_style().await.expect("Failed to get cursor style");
    assert_eq!(cursor, cursor::AUTO, "Cursor should be 'auto' in empty area");
    
    // Test 2: Cursor sobre el rectángulo debe ser 'grab'
    setup.hover_at(275.0, 250.0).await.expect("Failed to hover rectangle");
    let cursor = setup.get_cursor_style().await.expect("Failed to get cursor style");
    assert_eq!(cursor, cursor::GRAB, "Cursor should be 'grab' when hovering over shape");
    
    // Test 3: Cursor fuera del rectángulo debe volver a 'auto'
    setup.hover_at(400.0, 100.0).await.expect("Failed to hover empty area");
    let cursor = setup.get_cursor_style().await.expect("Failed to get cursor style");
    assert_eq!(cursor, cursor::AUTO, "Cursor should return to 'auto' outside shape");
    
    println!("✅ Shape hover cursor behavior works correctly");
}

#[tokio::test]
async fn test_cursor_changes_on_handle_hover() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear y seleccionar un rectángulo
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    setup.select_element(275.0, 250.0)
        .await
        .expect("Failed to select rectangle");
    
    // Tomar screenshot para ver los handles
    setup.screenshot("handles_visible")
        .await
        .expect("Failed to take screenshot");
    
    // Test cursors en diferentes handles (basado en nuestra implementación)
    // Handle superior izquierdo (nw-resize)
    setup.hover_at(200.0, 200.0).await.expect("Failed to hover top-left handle");
    let cursor = setup.get_cursor_style().await.expect("Failed to get cursor style");
    // Nota: Podría ser nw-resize o alguno de los resize cursors
    assert!(
        cursor.contains("resize") || cursor == cursor::NW_RESIZE,
        "Cursor should be resize type on top-left handle, got: {}", cursor
    );
    
    // Handle inferior derecho (se-resize)
    setup.hover_at(350.0, 300.0).await.expect("Failed to hover bottom-right handle");
    let cursor = setup.get_cursor_style().await.expect("Failed to get cursor style");
    assert!(
        cursor.contains("resize") || cursor == cursor::SE_RESIZE,
        "Cursor should be resize type on bottom-right handle, got: {}", cursor
    );
    
    // Handle lateral derecho (e-resize)
    setup.hover_at(350.0, 250.0).await.expect("Failed to hover right handle");
    let cursor = setup.get_cursor_style().await.expect("Failed to get cursor style");
    assert!(
        cursor.contains("resize") || cursor == cursor::E_RESIZE,
        "Cursor should be resize type on right handle, got: {}", cursor
    );
    
    // Volver al área vacía - cursor debe ser auto
    setup.hover_at(100.0, 100.0).await.expect("Failed to hover empty area");
    let cursor = setup.get_cursor_style().await.expect("Failed to get cursor style");
    assert_eq!(cursor, cursor::AUTO, "Cursor should return to 'auto' in empty area");
    
    println!("✅ Scale handle hover cursor behavior works correctly");
}

#[tokio::test]
async fn test_hover_detection_functions() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear un rectángulo
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    // Test de la función ecs_detect_shape_hover
    let shape_hover_result = setup.page.evaluate(
        "ecs_detect_shape_hover(275, 250)", // Centro del rectángulo
        serde_json::Value::Null
    ).await.expect("Failed to call ecs_detect_shape_hover");
    
    assert!(
        shape_hover_result.as_bool().unwrap_or(false),
        "ecs_detect_shape_hover should return true when hovering over shape"
    );
    
    // Test de la función ecs_detect_shape_hover fuera del rectángulo
    let no_shape_hover = setup.page.evaluate(
        "ecs_detect_shape_hover(100, 100)", // Área vacía
        serde_json::Value::Null
    ).await.expect("Failed to call ecs_detect_shape_hover");
    
    assert!(
        !no_shape_hover.as_bool().unwrap_or(true),
        "ecs_detect_shape_hover should return false when not hovering over shape"
    );
    
    // Seleccionar el rectángulo para probar handle detection
    setup.select_element(275.0, 250.0)
        .await
        .expect("Failed to select rectangle");
    
    // Test de la función ecs_detect_handle_hover en esquina superior izquierda
    let handle_hover_result = setup.page.evaluate(
        "ecs_detect_handle_hover(200, 200)", // Esquina superior izquierda
        serde_json::Value::Null
    ).await.expect("Failed to call ecs_detect_handle_hover");
    
    // La función debe retornar información del handle o null
    println!("Handle hover result: {:#?}", handle_hover_result);
    
    // Test handle hover fuera del área de handles
    let no_handle_hover = setup.page.evaluate(
        "ecs_detect_handle_hover(100, 100)", // Área vacía
        serde_json::Value::Null
    ).await.expect("Failed to call ecs_detect_handle_hover");
    
    // Debe ser null cuando no hay handle
    assert!(
        no_handle_hover.is_null(),
        "ecs_detect_handle_hover should return null when not hovering over handle"
    );
    
    println!("✅ WASM hover detection functions work correctly");
}

#[tokio::test] 
async fn test_hover_priority_handles_over_shapes() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear y seleccionar rectángulo
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    setup.select_element(275.0, 250.0)
        .await
        .expect("Failed to select rectangle");
    
    // Test prioridad: handle debe tener prioridad sobre shape
    // Hover en esquina donde hay tanto handle como shape
    setup.hover_at(200.0, 200.0).await.expect("Failed to hover corner");
    
    let cursor = setup.get_cursor_style().await.expect("Failed to get cursor style");
    
    // El cursor debe ser de resize (handle) no de grab (shape)
    assert!(
        cursor.contains("resize") && cursor != cursor::GRAB,
        "Handle cursor should have priority over shape cursor, got: {}", cursor
    );
    
    // Test que en el centro del shape (sin handles cerca) sea grab
    setup.hover_at(275.0, 250.0).await.expect("Failed to hover center");
    let cursor = setup.get_cursor_style().await.expect("Failed to get cursor style");
    assert_eq!(cursor, cursor::GRAB, "Center of selected shape should show grab cursor");
    
    println!("✅ Handle priority over shape hover works correctly");
}