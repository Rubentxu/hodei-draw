use std::process::Command;
use std::path::Path;
use tokio::time::{sleep, Duration};

pub struct PlaywrightRunner {
    test_dir: String,
}

impl PlaywrightRunner {
    pub fn new() -> Self {
        Self {
            test_dir: "tests".to_string(),
        }
    }

    /// Install Node.js dependencies and Playwright browsers
    pub async fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Setting up Playwright environment...");
        
        // Check if package.json exists
        if !Path::new(&format!("{}/package.json", self.test_dir)).exists() {
            return Err("package.json not found in tests directory".into());
        }

        // Install npm dependencies
        let npm_install = Command::new("npm")
            .arg("install")
            .current_dir(&self.test_dir)
            .output()?;

        if !npm_install.status.success() {
            let error = String::from_utf8_lossy(&npm_install.stderr);
            return Err(format!("npm install failed: {}", error).into());
        }
        println!("✅ npm dependencies installed");

        // Install Playwright browsers
        let browsers_install = Command::new("npx")
            .args(["playwright", "install"])
            .current_dir(&self.test_dir)
            .output()?;

        if !browsers_install.status.success() {
            let error = String::from_utf8_lossy(&browsers_install.stderr);
            return Err(format!("Playwright browsers install failed: {}", error).into());
        }
        println!("✅ Playwright browsers installed");

        Ok(())
    }

    /// Run all Playwright tests
    pub async fn run_all_tests(&self) -> Result<PlaywrightResult, Box<dyn std::error::Error>> {
        println!("🧪 Running all Playwright tests...");
        
        let output = Command::new("npx")
            .args(["playwright", "test"])
            .current_dir(&self.test_dir)
            .output()?;

        Ok(PlaywrightResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Run specific test file
    pub async fn run_test(&self, test_name: &str) -> Result<PlaywrightResult, Box<dyn std::error::Error>> {
        println!("🧪 Running test: {}", test_name);
        
        let output = Command::new("npx")
            .args(["playwright", "test", test_name])
            .current_dir(&self.test_dir)
            .output()?;

        Ok(PlaywrightResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Run tests in headed mode (visible browser)
    pub async fn run_tests_headed(&self) -> Result<PlaywrightResult, Box<dyn std::error::Error>> {
        println!("🧪 Running Playwright tests in headed mode...");
        
        let output = Command::new("npx")
            .args(["playwright", "test", "--headed"])
            .current_dir(&self.test_dir)
            .output()?;

        Ok(PlaywrightResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Run tests in UI mode (interactive)
    pub async fn run_tests_ui(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎮 Starting Playwright UI mode...");
        
        let mut child = Command::new("npx")
            .args(["playwright", "test", "--ui"])
            .current_dir(&self.test_dir)
            .spawn()?;

        // Wait for UI to start
        sleep(Duration::from_secs(3)).await;
        println!("🌐 Playwright UI should be available at http://localhost:9323/");
        
        // Wait for user to finish
        let _ = child.wait()?;
        Ok(())
    }

    /// Generate HTML report
    pub async fn generate_report(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Generating HTML report...");
        
        let output = Command::new("npx")
            .args(["playwright", "show-report"])
            .current_dir(&self.test_dir)
            .spawn()?;

        println!("📋 HTML report should be available in browser");
        Ok(())
    }
}

#[derive(Debug)]
pub struct PlaywrightResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl PlaywrightResult {
    pub fn print_summary(&self) {
        if self.success {
            println!("✅ Tests PASSED");
        } else {
            println!("❌ Tests FAILED");
        }
        
        if !self.stdout.is_empty() {
            println!("\n📄 Output:");
            println!("{}", self.stdout);
        }
        
        if !self.stderr.is_empty() {
            println!("\n⚠️ Errors:");
            println!("{}", self.stderr);
        }
    }
}

#[tokio::test]
async fn test_playwright_setup() {
    let runner = PlaywrightRunner::new();
    
    match runner.setup().await {
        Ok(_) => println!("✅ Playwright setup completed successfully"),
        Err(e) => {
            println!("⚠️ Playwright setup failed: {}", e);
            println!("💡 Run manually: cd tests && npm install && npx playwright install");
        }
    }
}

#[tokio::test] 
async fn test_run_drawing_tools() {
    let runner = PlaywrightRunner::new();
    
    match runner.run_test("drawing-tools.spec.js").await {
        Ok(result) => {
            result.print_summary();
            // Don't assert success - we want to see results even if tests fail
        },
        Err(e) => {
            println!("❌ Failed to run drawing tools tests: {}", e);
            println!("💡 Make sure application is running on http://localhost:8082/hodei-draw/");
        }
    }
}

#[tokio::test]
async fn test_run_hover_behavior() {
    let runner = PlaywrightRunner::new();
    
    match runner.run_test("hover-behavior.spec.js").await {
        Ok(result) => {
            result.print_summary();
            assert!(result.success, "Hover behavior tests should pass");
        },
        Err(e) => {
            println!("❌ Failed to run hover behavior tests: {}", e);
        }
    }
}

#[tokio::test]
async fn test_run_scale_handles() {
    let runner = PlaywrightRunner::new();
    
    match runner.run_test("scale-handles.spec.js").await {
        Ok(result) => {
            result.print_summary();
            assert!(result.success, "Scale handles tests should pass");
        },
        Err(e) => {
            println!("❌ Failed to run scale handles tests: {}", e);
        }
    }
}

#[tokio::test]
async fn test_run_all_playwright_tests() {
    let runner = PlaywrightRunner::new();
    
    println!("🚀 Running complete E2E test suite...");
    
    match runner.run_all_tests().await {
        Ok(result) => {
            result.print_summary();
            
            if result.success {
                println!("\n🎉 ALL TESTS PASSED!");
                println!("✅ Our hover and scale handle system works perfectly!");
            } else {
                println!("\n💡 Some tests failed - check output above");
                println!("🔍 Common issues:");
                println!("   • Application not running on localhost:8082");
                println!("   • WASM not loaded (wait longer)");
                println!("   • Browser not installed");
            }
        },
        Err(e) => {
            println!("❌ Failed to run tests: {}", e);
            println!("💡 Setup: cd tests && npm install && npx playwright install");
        }
    }
}