use playwright::{api::*, Playwright};
use tokio::time::{sleep, Duration};
use std::path::PathBuf;

pub struct TestSetup {
    pub browser: Browser,
    pub context: BrowserContext,
    pub page: Page,
}

impl TestSetup {
    /// Inicializa Playwright con Chromium y navega a la aplicación
    pub async fn new() -> Result<Self, Error> {
        let playwright = Playwright::initialize().await?;
        let browser_type = playwright.chromium();
        
        let browser = browser_type
            .launcher()
            .headless(false) // Visual para debugging
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-web-security")
            .launch()
            .await?;

        let context = browser
            .context_builder()
            .viewport(1200, 800)
            .build()
            .await?;

        let page = context.new_page().await?;
        
        // Navegar a la aplicación local
        page.goto_builder("http://localhost:8082/hodei-draw/")
            .goto()
            .await?;
        
        // Esperar a que se cargue la aplicación WASM
        page.wait_for_load_state_builder()
            .state(LoadState::Networkidle)
            .wait_for_load_state()
            .await?;
        sleep(Duration::from_millis(2000)).await; // Tiempo adicional para WASM

        Ok(TestSetup {
            browser,
            context,
            page,
        })
    }

    /// Crea un rectángulo usando las funciones WASM
    pub async fn create_rectangle(&self, x: f32, y: f32, width: f32, height: f32) -> Result<(), Error> {
        self.page.evaluate(
            &format!("ecs_create_rect({}, {}, {}, {})", x, y, width, height),
            serde_json::Value::Null
        ).await?;
        
        // Pequeña pausa para el render
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Selecciona un elemento haciendo clic en las coordenadas
    pub async fn select_element(&self, x: f32, y: f32) -> Result<(), Error> {
        self.page.evaluate(
            &format!("ecs_pointer_down({}, {})", x, y),
            serde_json::Value::Null
        ).await?;
        
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Obtiene el cursor actual del elemento canvas
    pub async fn get_cursor_style(&self) -> Result<String, Error> {
        let cursor = self.page.evaluate(
            "window.getComputedStyle(document.querySelector('canvas')).cursor",
            serde_json::Value::Null
        ).await?;
        
        Ok(cursor.as_str().unwrap_or("auto").to_string())
    }

    /// Mueve el mouse sobre coordenadas específicas
    pub async fn hover_at(&self, x: f32, y: f32) -> Result<(), Error> {
        self.page.mouse().r#move(x.into(), y.into()).await?;
        sleep(Duration::from_millis(100)).await; // Tiempo para que se aplique hover
        Ok(())
    }

    /// Toma screenshot para comparación visual
    pub async fn screenshot(&self, name: &str) -> Result<Vec<u8>, Error> {
        self.page.screenshot_builder()
            .path(format!("tests/screenshots/{}.png", name))
            .full_page(true)
            .screenshot()
            .await
    }

    /// Verifica que existan las funciones WASM esperadas
    pub async fn verify_wasm_functions(&self) -> Result<(), Error> {
        let functions = self.page.evaluate(
            r#"
            [
                'ecs_create_rect',
                'ecs_pointer_down', 
                'ecs_detect_handle_hover',
                'ecs_detect_shape_hover'
            ].map(fn => ({ name: fn, available: typeof window[fn] === 'function' }))
            "#,
            serde_json::Value::Null
        ).await?;

        println!("WASM Functions: {:#?}", functions);
        Ok(())
    }
}

impl Drop for TestSetup {
    fn drop(&mut self) {
        // Cleanup se maneja automáticamente por Playwright
    }
}

/// Utileades para testing de cursor
pub mod cursor {
    pub const AUTO: &str = "auto";
    pub const POINTER: &str = "pointer";
    pub const GRAB: &str = "grab";
    pub const GRABBING: &str = "grabbing";
    pub const NW_RESIZE: &str = "nw-resize";
    pub const SE_RESIZE: &str = "se-resize";
    pub const NE_RESIZE: &str = "ne-resize";
    pub const SW_RESIZE: &str = "sw-resize";
    pub const N_RESIZE: &str = "n-resize";
    pub const S_RESIZE: &str = "s-resize";
    pub const E_RESIZE: &str = "e-resize";
    pub const W_RESIZE: &str = "w-resize";
}

/// Constantes para coordenadas de test
pub mod coords {
    pub const CANVAS_WIDTH: f32 = 800.0;
    pub const CANVAS_HEIGHT: f32 = 600.0;
    
    // Posiciones comunes para testing
    pub const CENTER: (f32, f32) = (CANVAS_WIDTH / 2.0, CANVAS_HEIGHT / 2.0);
    pub const TOP_LEFT: (f32, f32) = (100.0, 100.0);
    pub const BOTTOM_RIGHT: (f32, f32) = (700.0, 500.0);
}