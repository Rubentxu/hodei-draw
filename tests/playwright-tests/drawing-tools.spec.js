const { test, expect } = require('@playwright/test');

test.describe('Drawing Tools', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/hodei-draw/');
    // Wait for WASM to load
    await page.waitForTimeout(2000);
  });

  test('should create rectangle using WASM function', async ({ page }) => {
    // Create rectangle using WASM function directly
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
    });

    // Take screenshot to verify rectangle was created
    await expect(page).toHaveScreenshot('rectangle-created.png');

    // Verify canvas has content (not empty)
    const hasContent = await page.evaluate(() => {
      const canvas = document.querySelector('canvas');
      const ctx = canvas.getContext('2d');
      const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const data = imageData.data;
      
      // Check for non-white pixels (drawing content)
      for (let i = 0; i < data.length; i += 4) {
        if (data[i] !== 255 || data[i+1] !== 255 || data[i+2] !== 255) {
          return true;
        }
      }
      return false;
    });

    expect(hasContent).toBe(true);
  });

  test('should create multiple shapes', async ({ page }) => {
    // Create multiple shapes using WASM functions
    await page.evaluate(() => {
      window.ecs_create_rect(100, 100, 100, 80);
      window.ecs_create_ellipse(300, 150, 50, 50);
      window.ecs_create_line(100, 250, 200, 300);
    });

    await expect(page).toHaveScreenshot('multiple-shapes.png');
  });

  test('should verify WASM functions are available', async ({ page }) => {
    // Verify all expected WASM functions exist
    const functions = await page.evaluate(() => {
      return [
        'ecs_create_rect',
        'ecs_create_ellipse', 
        'ecs_create_line',
        'ecs_pointer_down',
        'ecs_detect_handle_hover',
        'ecs_detect_shape_hover'
      ].map(fn => ({
        name: fn,
        available: typeof window[fn] === 'function'
      }));
    });

    console.log('WASM Functions:', functions);

    // All functions should be available
    functions.forEach(func => {
      expect(func.available).toBe(true);
    });
  });
});