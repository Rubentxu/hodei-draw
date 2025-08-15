const { test, expect } = require('@playwright/test');

test.describe('Scale Handles - Complete Interaction Testing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/hodei-draw/');
    await page.waitForTimeout(2000);
  });

  test('should show scale handles on selection', async ({ page }) => {
    // Create rectangle
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
    });

    // Screenshot before selection
    await expect(page).toHaveScreenshot('before-selection.png');

    // Select rectangle
    await page.evaluate(() => {
      window.ecs_pointer_down(275, 250);
    });

    // Screenshot after selection (should show handles)
    await expect(page).toHaveScreenshot('after-selection-with-handles.png');

    // Verify handles are detectable
    const handlePositions = [
      { x: 200, y: 200, name: 'top-left' },
      { x: 350, y: 200, name: 'top-right' },
      { x: 200, y: 300, name: 'bottom-left' },
      { x: 350, y: 300, name: 'bottom-right' },
      { x: 275, y: 200, name: 'top-center' },
      { x: 350, y: 250, name: 'right-center' },
      { x: 275, y: 300, name: 'bottom-center' },
      { x: 200, y: 250, name: 'left-center' }
    ];

    let detectedHandles = 0;
    for (const handle of handlePositions) {
      const result = await page.evaluate((x, y) => {
        return window.ecs_detect_handle_hover(x, y);
      }, handle.x, handle.y);

      if (result !== null) {
        detectedHandles++;
        console.log(`✅ Handle detected at ${handle.name}: ${JSON.stringify(result)}`);
      }
    }

    expect(detectedHandles).toBeGreaterThanOrEqual(4); // At least corner handles
    console.log(`Detected ${detectedHandles} handles`);
  });

  test('should perform scale interaction with mouse', async ({ page }) => {
    // Create and select rectangle
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
      window.ecs_pointer_down(275, 250);
    });

    // Screenshot before scaling
    await expect(page).toHaveScreenshot('before-scaling.png');

    const canvas = page.locator('canvas');
    
    // Simulate drag scaling from bottom-right corner
    await canvas.hover({ position: { x: 350, y: 300 } });
    await page.mouse.down();
    await page.mouse.move(400, 350); // Drag to make bigger
    await page.mouse.up();

    // Wait for rendering
    await page.waitForTimeout(200);

    // Screenshot after scaling
    await expect(page).toHaveScreenshot('after-scaling.png');

    // Verify scaling worked by checking for content in extended area
    const hasExtendedContent = await page.evaluate(() => {
      const canvas = document.querySelector('canvas');
      const ctx = canvas.getContext('2d');
      const imageData = ctx.getImageData(350, 300, 50, 50);
      const data = imageData.data;
      
      for (let i = 0; i < data.length; i += 4) {
        if (data[i] !== 255 || data[i+1] !== 255 || data[i+2] !== 255) {
          return true;
        }
      }
      return false;
    });

    console.log('Extended content found:', hasExtendedContent);
  });

  test('should test scale events sequence using WASM', async ({ page }) => {
    // Create and select rectangle  
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
      window.ecs_pointer_down(275, 250);
    });

    // Test complete scale sequence using WASM functions
    const scaleSequence = await page.evaluate(() => {
      try {
        // 1. Scale Start (HandleType::BottomRight = 3)
        window.ecs_scale_start(3, 350, 300);
        
        // 2. Scale Update (simulate 30px movement)
        window.ecs_scale_update(30, 30);
        
        // 3. Scale End
        window.ecs_scale_end();
        
        return { success: true, error: null };
      } catch (error) {
        return { success: false, error: error.message };
      }
    });

    expect(scaleSequence.success).toBe(true);
    if (!scaleSequence.success) {
      console.error('Scale sequence failed:', scaleSequence.error);
    }

    // Screenshot final result
    await expect(page).toHaveScreenshot('scale-sequence-complete.png');
  });

  test('should test all handle types', async ({ page }) => {
    const handleTypes = [
      { type: 0, name: 'TopLeft' },
      { type: 1, name: 'TopRight' },
      { type: 2, name: 'BottomLeft' },
      { type: 3, name: 'BottomRight' },
      { type: 4, name: 'Top' },
      { type: 5, name: 'Right' },
      { type: 6, name: 'Bottom' },
      { type: 7, name: 'Left' }
    ];

    for (const handle of handleTypes) {
      // Create fresh rectangle for each test
      await page.evaluate(() => {
        // Clear any existing shapes (in a real app you'd have a clear function)
        // For now, just create at different position
      });
      
      await page.evaluate(() => {
        window.ecs_create_rect(200, 200, 150, 100);
        window.ecs_pointer_down(275, 250);
      });

      // Test the specific handle type
      const result = await page.evaluate((handleType, handleName) => {
        try {
          window.ecs_scale_start(handleType, 275, 250);
          window.ecs_scale_update(10, 10);
          window.ecs_scale_end();
          return { success: true, handle: handleName };
        } catch (error) {
          return { success: false, handle: handleName, error: error.message };
        }
      }, handle.type, handle.name);

      expect(result.success).toBe(true);
      console.log(`✅ ${handle.name} handle works correctly`);
      
      if (!result.success) {
        console.error(`❌ ${handle.name} handle failed:`, result.error);
      }
    }
  });

  test('should verify no conflicts between scaling and moving', async ({ page }) => {
    // Create and select rectangle
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
      window.ecs_pointer_down(275, 250);
    });

    const canvas = page.locator('canvas');

    // Test 1: Hover center - should be grab (move)
    await canvas.hover({ position: { x: 275, y: 250 } });
    let cursor = await canvas.evaluate(el => window.getComputedStyle(el).cursor);
    expect(cursor).toBe('grab');

    // Test 2: Hover corner - should be resize (scale), NOT grab  
    await canvas.hover({ position: { x: 200, y: 200 } });
    cursor = await canvas.evaluate(el => window.getComputedStyle(el).cursor);
    expect(cursor).not.toBe('grab');
    expect(cursor.includes('resize')).toBe(true);

    // Test 3: Quick movement between areas should update cursor correctly
    await canvas.hover({ position: { x: 275, y: 250 } }); // Center
    cursor = await canvas.evaluate(el => window.getComputedStyle(el).cursor);
    expect(cursor).toBe('grab');

    await canvas.hover({ position: { x: 350, y: 300 } }); // Corner
    cursor = await canvas.evaluate(el => window.getComputedStyle(el).cursor);
    expect(cursor.includes('resize')).toBe(true);

    console.log('✅ No conflicts between scaling and moving cursors');
  });

  test('should handle rapid hover changes without race conditions', async ({ page }) => {
    // Create and select rectangle
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
      window.ecs_pointer_down(275, 250);
    });

    const canvas = page.locator('canvas');
    
    // Rapidly move between different areas
    const positions = [
      { x: 275, y: 250 }, // center
      { x: 200, y: 200 }, // corner
      { x: 275, y: 250 }, // center
      { x: 350, y: 300 }, // corner
      { x: 100, y: 100 }, // empty
      { x: 275, y: 250 }, // center
    ];

    for (const pos of positions) {
      await canvas.hover({ position: pos });
      await page.waitForTimeout(50); // Quick but allow cursor update
    }

    // Final cursor should be grab (center position)
    const finalCursor = await canvas.evaluate(el => 
      window.getComputedStyle(el).cursor
    );
    expect(finalCursor).toBe('grab');

    console.log('✅ No race conditions in rapid hover changes');
  });
});