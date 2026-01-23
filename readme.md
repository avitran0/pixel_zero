# Pixel Zero

A low-level graphics system with a 2D sprite-based API for embedded Linux systems.

## Features

- Direct DRM/KMS rendering
- OpenGL ES support  
- 2D sprite-based graphics API (320x240 virtual framebuffer, GBA-style)
- Bitmap font rendering
- Automatic letterboxing for any screen size

## System Requirements

Fedora libs: `libdrm-devel mesa-libgbm-devel mesa-libEGL-devel`

## Graphics API

See [GRAPHICS_API.md](GRAPHICS_API.md) for complete documentation of the 2D sprite rendering API.
