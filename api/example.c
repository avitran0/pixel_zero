/**
 * Example C program using the pixel_zero API
 * 
 * Demonstrates basic sprite drawing and text rendering.
 */

#include "pixel_zero.h"
#include <stdio.h>
#include <unistd.h>

int main(void) {
    // Initialize graphics
    GraphicsContextHandle graphics = graphics_context_load();
    if (!graphics) {
        fprintf(stderr, "Failed to load graphics context\n");
        return 1;
    }
    
    // Create input handler
    InputHandle input = input_new();
    
    // Create a simple test sprite (16x16 red square with white border)
    SpriteHandle sprite = sprite_new(16, 16);
    for (uint32_t y = 0; y < 16; y++) {
        for (uint32_t x = 0; x < 16; x++) {
            if (x == 0 || x == 15 || y == 0 || y == 15) {
                sprite_set_pixel(sprite, x, y, 255, 255, 255, 255); // White
            } else {
                sprite_set_pixel(sprite, x, y, 255, 0, 0, 255); // Red
            }
        }
    }
    
    // Create default font
    BitmapFontHandle font = bitmap_font_default();
    
    // Main loop (run for 5 seconds or until A button pressed)
    int running = 1;
    int frames = 0;
    while (running && frames < 300) { // 5 seconds at 60fps
        // Update input
        input_update(input);
        
        // Check for exit
        if (input_just_pressed(input, BUTTON_A)) {
            printf("Exiting\n");
            running = 0;
        }
        
        // Clear framebuffer to dark blue
        graphics_clear_framebuffer(graphics, 50, 50, 100, 255);
        
        // Draw test sprites
        graphics_draw_sprite(graphics, sprite, 10, 10);
        graphics_draw_sprite(graphics, sprite, 50, 50);
        graphics_draw_sprite(graphics, sprite, 150, 100);
        
        // Draw text
        graphics_draw_text(graphics, font, "Hello, Pixel Zero!", 10, 150, 255, 255, 255, 255);
        graphics_draw_text(graphics, font, "Press A to exit", 10, 170, 0, 255, 0, 255);
        
        // Present to screen
        if (graphics_present(graphics) != 0) {
            fprintf(stderr, "Failed to present framebuffer\n");
            break;
        }
        
        frames++;
        usleep(16666); // ~60fps
    }
    
    // Cleanup
    bitmap_font_free(font);
    sprite_free(sprite);
    input_free(input);
    graphics_context_free(graphics);
    
    return 0;
}
