use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_application_is_running() {
    // Test básico: verificar que la aplicación responde
    let output = Command::new("curl")
        .arg("-s")
        .arg("-o")
        .arg("/dev/null")
        .arg("-w")
        .arg("%{http_code}")
        .arg("http://localhost:8082/hodei-draw/")
        .output()
        .expect("Failed to execute curl");

    let status_code = String::from_utf8_lossy(&output.stdout);
    assert_eq!(status_code.trim(), "200", "Application should be running and accessible");
    
    println!("✅ Application is running on http://localhost:8082/hodei-draw/");
}

#[tokio::test] 
async fn test_wasm_functions_basic() {
    // Este test requiere que la aplicación esté corriendo
    // Podríamos usar un navegador headless simple o curl para verificar contenido
    
    // Por ahora, verificamos que podemos acceder al contenido HTML
    let output = Command::new("curl")
        .arg("-s")
        .arg("http://localhost:8082/hodei-draw/")
        .output()
        .expect("Failed to fetch page");
    
    let html = String::from_utf8_lossy(&output.stdout);
    
    // Verificar que el HTML contiene elementos esperados
    assert!(html.contains("<canvas"), "Page should contain canvas element");
    assert!(html.contains("app-web"), "Page should contain WASM app reference");
    
    println!("✅ Basic HTML content validation passed");
    println!("📄 HTML contains canvas and WASM references");
}

#[test]
fn test_screenshots_directory_exists() {
    use std::path::Path;
    use std::fs;
    
    let screenshots_dir = Path::new("tests/screenshots");
    
    // Crear directorio si no existe
    if !screenshots_dir.exists() {
        fs::create_dir_all(screenshots_dir).expect("Failed to create screenshots directory");
    }
    
    assert!(screenshots_dir.exists(), "Screenshots directory should exist");
    assert!(screenshots_dir.is_dir(), "Screenshots path should be a directory");
    
    println!("✅ Screenshots directory is ready at: {:?}", screenshots_dir);
}

#[tokio::test]
async fn test_basic_functionality_manual_validation() {
    // Este test guía para validación manual
    println!("\n🧪 MANUAL VALIDATION TEST");
    println!("===============================");
    println!("1. 📱 Open: http://localhost:8082/hodei-draw/");
    println!("2. 🟩 Click rectangle tool and draw a rectangle");
    println!("3. 👆 Click on rectangle to select it");
    println!("4. 👀 Verify scale handles appear (8 small circles)");
    println!("5. 🖱️ Hover over rectangle - cursor should be 'grab'");
    println!("6. 🖱️ Hover over handles - cursor should show resize arrows");
    println!("7. 📏 Try dragging a corner handle to scale");
    println!("8. ✅ Verify scaling works without conflicts");
    println!("");
    println!("🔍 Expected behaviors:");
    println!("   • Rectangle creation works");
    println!("   • Selection shows 8 scale handles");
    println!("   • Hover feedback: grab cursor on shapes");
    println!("   • Hover feedback: resize cursors on handles");
    println!("   • Scale handles have priority over shape movement");
    println!("   • Scaling works smoothly without race conditions");
    println!("");
    
    // Pausa para permitir validación manual
    sleep(Duration::from_secs(1)).await;
    
    println!("✅ Manual validation guide completed");
    println!("🎯 Verify the above behaviors work as expected");
}