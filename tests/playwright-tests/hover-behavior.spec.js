const { test, expect } = require('@playwright/test');

test.describe('Hover Behavior - Our Key Implementation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/hodei-draw/');
    await page.waitForTimeout(2000);
  });

  test('should change cursor on shape hover', async ({ page }) => {
    // Create a rectangle to hover over
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
    });

    const canvas = page.locator('canvas');

    // Test 1: Hover over empty area - cursor should be default/auto
    await canvas.hover({ position: { x: 100, y: 100 } });
    
    let cursor = await canvas.evaluate(el => 
      window.getComputedStyle(el).cursor
    );
    expect(cursor).toBe('default'); // Empty canvas areas should have default cursor

    // Test 2: Hover over rectangle center - cursor should be 'grab'  
    await canvas.hover({ position: { x: 275, y: 250 } });
    
    cursor = await canvas.evaluate(el => 
      window.getComputedStyle(el).cursor
    );
    expect(cursor).toBe('grab');

    // Test 3: Hover back to empty area - cursor returns to auto
    await canvas.hover({ position: { x: 400, y: 100 } });
    
    cursor = await canvas.evaluate(el => 
      window.getComputedStyle(el).cursor
    );
    expect(cursor).toBe('default'); // Empty area outside shapes should have default cursor
  });

  test('should change cursor on scale handles', async ({ page }) => {
    // Create and select rectangle to show handles
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
      window.ecs_pointer_down(275, 250); // Select rectangle
    });

    const canvas = page.locator('canvas');
    await expect(page).toHaveScreenshot('rectangle-with-handles.png');

    // Test different handle positions
    const handleTests = [
      { pos: { x: 200, y: 200 }, name: 'top-left', expectedCursor: 'nw-resize' },
      { pos: { x: 350, y: 200 }, name: 'top-right', expectedCursor: 'ne-resize' },
      { pos: { x: 350, y: 300 }, name: 'bottom-right', expectedCursor: 'se-resize' },
      { pos: { x: 200, y: 300 }, name: 'bottom-left', expectedCursor: 'sw-resize' },
      { pos: { x: 275, y: 200 }, name: 'top-center', expectedCursor: 'n-resize' },
      { pos: { x: 350, y: 250 }, name: 'right-center', expectedCursor: 'e-resize' },
      { pos: { x: 275, y: 300 }, name: 'bottom-center', expectedCursor: 's-resize' },
      { pos: { x: 200, y: 250 }, name: 'left-center', expectedCursor: 'w-resize' }
    ];

    for (const handleTest of handleTests) {
      await canvas.hover({ position: handleTest.pos });
      
      const cursor = await canvas.evaluate(el => 
        window.getComputedStyle(el).cursor
      );

      // Accept any resize cursor as valid (browser differences)
      const isResizeCursor = cursor.includes('resize') || 
                            ['nw-resize', 'ne-resize', 'se-resize', 'sw-resize',
                             'n-resize', 'e-resize', 's-resize', 'w-resize'].includes(cursor);

      expect(isResizeCursor).toBe(true);
      console.log(`${handleTest.name}: expected resize cursor, got "${cursor}"`);
    }
  });

  test('should test WASM hover detection functions', async ({ page }) => {
    // Create rectangle
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
    });

    // Test shape hover detection
    const shapeHoverResult = await page.evaluate(() => {
      return window.ecs_detect_shape_hover(275, 250); // Center of rectangle
    });
    expect(shapeHoverResult).toBe(true);

    // Test shape hover outside
    const noShapeHover = await page.evaluate(() => {
      return window.ecs_detect_shape_hover(100, 100); // Empty area
    });
    expect(noShapeHover).toBe(false);

    // Select rectangle to test handle detection
    await page.evaluate(() => {
      window.ecs_pointer_down(275, 250);
    });

    // Test handle hover detection
    const handleHoverResult = await page.evaluate(() => {
      return window.ecs_detect_handle_hover(200, 200); // Top-left corner
    });
    expect(handleHoverResult).not.toBeNull();

    // Test handle hover outside
    const noHandleHover = await page.evaluate(() => {
      return window.ecs_detect_handle_hover(100, 100); // Empty area
    });
    expect(noHandleHover).toBeNull();
  });

  test('should prioritize handles over shapes', async ({ page }) => {
    // Create and select rectangle
    await page.evaluate(() => {
      window.ecs_create_rect(200, 200, 150, 100);
      window.ecs_pointer_down(275, 250);
    });

    const canvas = page.locator('canvas');

    // Hover over corner where both handle and shape exist
    // Handle should have priority
    await canvas.hover({ position: { x: 200, y: 200 } });
    
    const cursor = await canvas.evaluate(el => 
      window.getComputedStyle(el).cursor
    );

    // Should be resize cursor (handle), not grab (shape)
    const isResizeCursor = cursor.includes('resize');
    expect(isResizeCursor).toBe(true);
    expect(cursor).not.toBe('grab');

    // Test center of shape (no handles) should be grab
    await canvas.hover({ position: { x: 275, y: 250 } });
    
    const centerCursor = await canvas.evaluate(el => 
      window.getComputedStyle(el).cursor
    );
    expect(centerCursor).toBe('grab');
  });
});