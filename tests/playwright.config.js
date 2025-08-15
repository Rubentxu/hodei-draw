// @ts-check
const { defineConfig, devices } = require('@playwright/test');

module.exports = defineConfig({
  testDir: './playwright-tests',
  fullyParallel: false, // Run tests sequentially for drawing app testing
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker for drawing app testing
  reporter: 'html',
  
  use: {
    // Base URL for all tests
    baseURL: 'http://localhost:8082',
    
    // Collect trace for debugging
    trace: 'on-first-retry',
    
    // Take screenshots on failure
    screenshot: 'only-on-failure',
    
    // Record video on failure
    video: 'retain-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  // Run local dev server before tests (optional - we assume it's already running)
  // webServer: {
  //   command: 'cd ../apps/app-web && trunk serve --port 8082',
  //   port: 8082,
  //   reuseExistingServer: !process.env.CI,
  // },
});