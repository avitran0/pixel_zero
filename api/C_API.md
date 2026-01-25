# Pixel Zero C API

The pixel_zero API crate provides a C-compatible FFI (Foreign Function Interface) for use from C and other languages.

## Building

The crate is configured to build both as a Rust library (`rlib`) and a C-compatible dynamic library (`cdylib`):

```bash
cd api
cargo build --release
```

This will produce:
- `target/release/libapi.so` (or `.dylib` on macOS, `.dll` on Windows) - The C-compatible shared library
- `target/release/libapi.rlib` - The Rust library

## Using from C

### Include the Header

```c
#include "pixel_zero.h"
```

### Link Against the Library

```bash
gcc -o example example.c -L../target/release -lapi -Wl,-rpath,../target/release
```

### Example Program

See `example.c` for a complete working example demonstrating:
- Graphics context initialization
- Sprite creation and drawing
- Text rendering with bitmap fonts
- Input handling
- Proper cleanup

## API Overview

### Graphics Context

```c
// Initialize
GraphicsContextHandle graphics = graphics_context_load();

// Clear screen
graphics_clear_framebuffer(graphics, 0, 0, 0, 255);

// Draw sprite
graphics_draw_sprite(graphics, sprite, x, y);

// Draw text
graphics_draw_text(graphics, font, "Hello!", x, y, 255, 255, 255, 255);

// Present to screen
graphics_present(graphics);

// Cleanup
graphics_context_free(graphics);
```

### Sprites

```c
// Create new sprite
SpriteHandle sprite = sprite_new(16, 16);

// Set pixels
sprite_set_pixel(sprite, x, y, r, g, b, a);

// Get pixels
uint8_t r, g, b, a;
sprite_get_pixel(sprite, x, y, &r, &g, &b, &a);

// Cleanup
sprite_free(sprite);
```

### Bitmap Fonts

```c
// Create default font
BitmapFontHandle font = bitmap_font_default();

// Or create from sprite atlas
BitmapFontHandle font = bitmap_font_new(atlas_sprite, 8, 8);

// Measure text
uint32_t width = bitmap_font_measure_text(font, "Hello");

// Cleanup
bitmap_font_free(font);
```

### Input

```c
// Create input handler
InputHandle input = input_new();

// Update each frame
input_update(input);

// Check buttons
if (input_is_pressed(input, BUTTON_A)) {
    // Button A is held down
}

if (input_just_pressed(input, BUTTON_A)) {
    // Button A was just pressed this frame
}

// Cleanup
input_free(input);
```

## Memory Management

All resources must be explicitly freed using the corresponding `_free()` functions:
- `graphics_context_free()`
- `sprite_free()`
- `bitmap_font_free()`
- `input_free()`

Failing to free resources will result in memory leaks.

## Error Handling

- Functions that return pointers return `NULL` on failure
- Functions that return integers return `0` on success and `-1` on error
- Check return values and handle errors appropriately

## Thread Safety

The API is **not thread-safe**. All API calls should be made from the same thread.

## Platform Support

Currently supports Linux with DRM/KMS and OpenGL ES 3.2. Requires:
- libdrm
- libgbm
- libEGL

## Complete Header Documentation

See `pixel_zero.h` for complete API documentation with detailed function descriptions and parameters.
