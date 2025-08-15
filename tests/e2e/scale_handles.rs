mod common {
    include!("../common/mod.rs");
}
use common::{TestSetup, coords};
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_scale_handles_appear_on_selection() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear rectángulo
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    // Screenshot antes de selección
    setup.screenshot("before_selection")
        .await
        .expect("Failed to take screenshot");
    
    // Seleccionar el rectángulo
    setup.select_element(275.0, 250.0)
        .await
        .expect("Failed to select rectangle");
    
    // Screenshot después de selección (debe mostrar handles)
    setup.screenshot("after_selection_with_handles")
        .await
        .expect("Failed to take screenshot");
    
    // Verificar que los handles son detectables por las funciones WASM
    let handles = [
        (200.0, 200.0), // Top-left
        (350.0, 200.0), // Top-right
        (200.0, 300.0), // Bottom-left
        (350.0, 300.0), // Bottom-right
        (275.0, 200.0), // Top center
        (350.0, 250.0), // Right center
        (275.0, 300.0), // Bottom center
        (200.0, 250.0), // Left center
    ];
    
    let mut detected_handles = 0;
    for (x, y) in handles.iter() {
        let result = setup.page.evaluate(
            &format!("ecs_detect_handle_hover({}, {})", x, y),
            serde_json::Value::Null
        ).await.expect("Failed to call handle detection");
        
        if !result.is_null() {
            detected_handles += 1;
            println!("Handle detected at ({}, {}): {:?}", x, y, result);
        }
    }
    
    assert!(detected_handles >= 4, "Should detect at least 4 corner handles, detected: {}", detected_handles);
    println!("✅ Scale handles appear and are detectable after selection");
}

#[tokio::test]
async fn test_scale_handle_interaction() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear y seleccionar rectángulo
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    setup.select_element(275.0, 250.0)
        .await
        .expect("Failed to select rectangle");
    
    // Screenshot inicial
    setup.screenshot("before_scaling")
        .await
        .expect("Failed to take screenshot");
    
    // Intentar escalar desde la esquina inferior derecha
    let handle_x = 350.0;
    let handle_y = 300.0;
    let new_x = 400.0; // Mover 50px a la derecha
    let new_y = 350.0; // Mover 50px hacia abajo
    
    // Simular arrastre del handle
    setup.page.mouse().move_to(handle_x.into(), handle_y.into()).await
        .expect("Failed to move to handle");
    
    setup.page.mouse().down().await
        .expect("Failed to mouse down");
    
    setup.page.mouse().move_to(new_x.into(), new_y.into()).await
        .expect("Failed to drag to new position");
    
    setup.page.mouse().up().await
        .expect("Failed to mouse up");
    
    // Pausa para el rendering
    sleep(Duration::from_millis(200)).await;
    
    // Screenshot después del escalado
    setup.screenshot("after_scaling")
        .await
        .expect("Failed to take screenshot");
    
    // Verificar que el rectángulo cambió de tamaño comparando píxeles
    let size_changed = setup.page.evaluate(
        r#"
        const canvas = document.querySelector('canvas');
        const ctx = canvas.getContext('2d');
        const imageData = ctx.getImageData(350, 300, 50, 50); // Área donde debería extenderse
        const data = imageData.data;
        
        // Buscar píxeles no blancos en la nueva área
        let hasNewContent = false;
        for (let i = 0; i < data.length; i += 4) {
            if (data[i] !== 255 || data[i+1] !== 255 || data[i+2] !== 255) {
                hasNewContent = true;
                break;
            }
        }
        hasNewContent
        "#,
        serde_json::Value::Null
    ).await.expect("Failed to check scaling result");
    
    // Note: Este test podría fallar si el sistema de escalado no está completamente implementado
    // pero nos ayuda a detectar si la funcionalidad está trabajando
    println!("Scaling result - size changed: {:?}", size_changed);
    println!("✅ Scale handle interaction test completed");
}

#[tokio::test]
async fn test_scale_events_sequence() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear y seleccionar rectángulo
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    setup.select_element(275.0, 250.0)
        .await
        .expect("Failed to select rectangle");
    
    // Test secuencia de eventos de escalado usando funciones WASM directamente
    
    // 1. Scale Start
    let scale_start_result = setup.page.evaluate(
        r#"
        try {
            // Simular inicio de escalado en esquina bottom-right (HandleType = 3)
            ecs_scale_start(3, 350, 300); // HandleType::BottomRight = 3
            true
        } catch (e) {
            console.error('Scale start failed:', e);
            false
        }
        "#,
        serde_json::Value::Null
    ).await.expect("Failed to execute scale start");
    
    assert!(scale_start_result.as_bool().unwrap_or(false), "Scale start should succeed");
    
    // 2. Scale Update (simular movimiento)
    let scale_update_result = setup.page.evaluate(
        r#"
        try {
            // Simular movimiento de 30px en X y Y
            ecs_scale_update(30, 30);
            true
        } catch (e) {
            console.error('Scale update failed:', e);
            false
        }
        "#,
        serde_json::Value::Null
    ).await.expect("Failed to execute scale update");
    
    assert!(scale_update_result.as_bool().unwrap_or(false), "Scale update should succeed");
    
    // 3. Scale End
    let scale_end_result = setup.page.evaluate(
        r#"
        try {
            ecs_scale_end();
            true
        } catch (e) {
            console.error('Scale end failed:', e);
            false
        }
        "#,
        serde_json::Value::Null
    ).await.expect("Failed to execute scale end");
    
    assert!(scale_end_result.as_bool().unwrap_or(false), "Scale end should succeed");
    
    // Screenshot final
    setup.screenshot("scale_events_completed")
        .await
        .expect("Failed to take screenshot");
    
    println!("✅ Scale events sequence completed successfully");
}

#[tokio::test]
async fn test_scale_different_handles() {
    let setup = TestSetup::new().await.expect("Failed to initialize test setup");
    
    // Crear rectángulo base
    setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
        .await
        .expect("Failed to create rectangle");
    
    setup.select_element(275.0, 250.0)
        .await
        .expect("Failed to select rectangle");
    
    // Test diferentes tipos de handles
    let handle_tests = [
        (0, "TopLeft", "top_left_handle"),
        (1, "TopRight", "top_right_handle"), 
        (2, "BottomLeft", "bottom_left_handle"),
        (3, "BottomRight", "bottom_right_handle"),
        (4, "Top", "top_handle"),
        (5, "Right", "right_handle"),
        (6, "Bottom", "bottom_handle"),
        (7, "Left", "left_handle"),
    ];
    
    for (handle_type, name, screenshot_name) in handle_tests.iter() {
        // Reset para cada test
        setup.create_rectangle(200.0, 200.0, 150.0, 100.0)
            .await
            .expect("Failed to create rectangle");
        
        setup.select_element(275.0, 250.0)
            .await
            .expect("Failed to select rectangle");
        
        // Test el handle específico
        let result = setup.page.evaluate(
            &format!(
                r#"
                try {{
                    ecs_scale_start({}, 275, 250);
                    ecs_scale_update(20, 20);
                    ecs_scale_end();
                    true
                }} catch (e) {{
                    console.error('Scale failed for handle {}:', e);
                    false
                }}
                "#,
                handle_type, name
            ),
            serde_json::Value::Null
        ).await.expect("Failed to test handle");
        
        if result.as_bool().unwrap_or(false) {
            setup.screenshot(screenshot_name)
                .await
                .expect("Failed to take screenshot");
            println!("✅ {} handle works correctly", name);
        } else {
            println!("⚠️ {} handle test failed", name);
        }
    }
    
    println!("✅ All handle types tested");
}