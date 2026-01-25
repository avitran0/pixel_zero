# Pixel Zero

A low-level graphics system with a 2D sprite-based API for embedded Linux systems.

## Features

- Direct DRM/KMS rendering
- OpenGL ES support  
- 2D sprite-based graphics API (320x240 virtual framebuffer, GBA-style)
- Bitmap font rendering
- Automatic letterboxing for any screen size
- **C-compatible FFI** for use from C and other languages

## System Requirements

Fedora libs: `libdrm-devel mesa-libgbm-devel mesa-libEGL-devel`

## Graphics API

See [GRAPHICS_API.md](GRAPHICS_API.md) for complete documentation of the 2D sprite rendering API.

## C API

The API crate provides a C-compatible FFI. See [api/C_API.md](api/C_API.md) for:
- Building instructions
- C header file (`api/pixel_zero.h`)
- Example C program (`api/example.c`)
- Complete API documentation

### Quick Start (C)

```c
#include "pixel_zero.h"

int main(void) {
    GraphicsContextHandle graphics = graphics_context_load();
    SpriteHandle sprite = sprite_new(16, 16);
    
    // Draw something...
    graphics_clear_framebuffer(graphics, 0, 0, 0, 255);
    graphics_draw_sprite(graphics, sprite, 10, 10);
    graphics_present(graphics);
    
    sprite_free(sprite);
    graphics_context_free(graphics);
    return 0;
}
```
