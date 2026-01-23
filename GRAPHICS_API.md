# 2D Sprite-Based Graphics API

This graphics API provides a simple, GBA-like 2D sprite rendering system with a fixed 320x240 framebuffer that is automatically letterboxed to fit any display size. All rendering is done using OpenGL ES for hardware acceleration.

## Features

- **Fixed Resolution**: 320x240 rendering target (GBA resolution)
- **Automatic Letterboxing**: Content is scaled and centered on any display size
- **Sprite Rendering**: Draw sprites with alpha transparency using OpenGL textures
- **Bitmap Font Support**: Render text using bitmap fonts
- **Hardware Accelerated**: All rendering is done via OpenGL ES primitives and shaders
- **Framebuffer Object**: Uses OpenGL FBO for off-screen 320x240 rendering

## Technical Details

- The rendering target is a 320x240 OpenGL framebuffer object (FBO)
- Sprites are uploaded as OpenGL textures and rendered as textured quads
- Text is rendered by drawing individual glyph sprites
- The FBO is then rendered to the screen with letterboxing and nearest-neighbor filtering
- Alpha blending is handled natively by OpenGL
- All rendering uses OpenGL ES 3.2 shaders

## Basic Usage

### Creating and Drawing Sprites

```rust
use api::graphics::{GraphicsContext, sprite::Sprite, color::Color};

// Create a sprite
let mut sprite = Sprite::new(16, 16);

// Set pixels
for y in 0..16 {
    for x in 0..16 {
        sprite.set_pixel(x, y, Color::RED);
    }
}

// Draw the sprite
graphics.draw_sprite(&sprite, 10, 10);
```

### Drawing Text

```rust
use api::graphics::{font::BitmapFont, color::Color};

let font = BitmapFont::default();
graphics.draw_text(&font, "Hello, World!", 10, 50, Color::WHITE);
```

### Clearing the Screen

```rust
use api::graphics::color::Color;

// Clear to a specific color
graphics.clear_framebuffer(Color::rgb(50, 50, 100));
```

### Complete Example

```rust
use api::graphics::{GraphicsContext, sprite::Sprite, font::BitmapFont, color::Color};

// Initialize graphics
let mut graphics = GraphicsContext::load().unwrap();

// Create a sprite
let mut sprite = Sprite::new(16, 16);
for y in 0..16 {
    for x in 0..16 {
        if x == 0 || x == 15 || y == 0 || y == 15 {
            sprite.set_pixel(x, y, Color::WHITE);
        } else {
            sprite.set_pixel(x, y, Color::RED);
        }
    }
}

// Create font
let font = BitmapFont::default();

// Main loop
loop {
    // Clear the framebuffer
    graphics.clear_framebuffer(Color::BLACK);
    
    // Draw sprites
    graphics.draw_sprite(&sprite, 10, 10);
    graphics.draw_sprite(&sprite, 50, 50);
    
    // Draw text
    graphics.draw_text(&font, "Hello, Pixel Zero!", 10, 150, Color::WHITE);
    
    // Present to screen (handles letterboxing automatically)
    graphics.present().unwrap();
}
```

## API Reference

### `GraphicsContext`

Main graphics context that manages the display and rendering.

#### Methods

- `load() -> Result<Self>` - Initialize the graphics context
- `clear_framebuffer(color: Color)` - Clear the virtual framebuffer to a color
- `draw_sprite(sprite: &Sprite, x: i32, y: i32)` - Draw a sprite at position (x, y)
- `draw_text(font: &BitmapFont, text: &str, x: i32, y: i32, color: Color)` - Draw text
- `framebuffer_size() -> (u32, u32)` - Get the virtual framebuffer dimensions (320x240)
- `present() -> Result<()>` - Present the framebuffer to the screen with letterboxing

### `Sprite`

2D sprite with pixel data.

#### Methods

- `new(width: u32, height: u32) -> Self` - Create a new empty sprite
- `from_pixels(width: u32, height: u32, pixels: Vec<Color>) -> Option<Self>` - Create from pixel data
- `width() -> u32` - Get sprite width
- `height() -> u32` - Get sprite height
- `get_pixel(x: u32, y: u32) -> Option<Color>` - Get pixel at position
- `set_pixel(x: u32, y: u32, color: Color)` - Set pixel at position
- `pixels() -> &[Color]` - Get reference to pixel data

### `BitmapFont`

Bitmap font for text rendering.

#### Methods

- `new(atlas: Sprite, glyph_width: u32, glyph_height: u32) -> Self` - Create from atlas
- `default() -> Self` - Create default 8x8 font
- `glyph_width() -> u32` - Get glyph width
- `glyph_height() -> u32` - Get glyph height
- `get_glyph(character: char) -> Option<Sprite>` - Get sprite for character
- `measure_text(text: &str) -> u32` - Measure text width in pixels

### `Color`

RGBA color representation.

#### Methods

- `rgb(r: u8, g: u8, b: u8) -> Self` - Create RGB color (alpha = 255)
- `rgba(r: u8, g: u8, b: u8, a: u8) -> Self` - Create RGBA color
- `r(), g(), b(), a() -> u8` - Get color components
- `as_f32_array() -> [f32; 4]` - Convert to float array (0.0-1.0)
- `as_u8_array() -> [u8; 4]` - Convert to u8 array

#### Constants

- `Color::BLACK` - Black (0, 0, 0)
- `Color::WHITE` - White (255, 255, 255)
- `Color::RED` - Red (255, 0, 0)
- `Color::GREEN` - Green (0, 255, 0)
- `Color::BLUE` - Blue (0, 0, 255)

## Performance Considerations

- Sprites are uploaded to GPU memory each time they're drawn
- For frequently used sprites, consider caching them on the CPU side
- The 320x240 resolution keeps rendering lightweight even without complex optimization
- Text rendering may create temporary textures for each glyph

## Custom Fonts

To use a custom bitmap font, create a sprite atlas with glyphs arranged in a grid:

```rust
// Load your font atlas sprite (e.g., from file or generated)
let font_atlas = Sprite::new(128, 48); // 16 chars per row, 6 rows for ASCII 32-126

// Create font with 8x8 glyphs
let font = BitmapFont::new(font_atlas, 8, 8);

// Use the font
graphics.draw_text(&font, "Custom Font!", 10, 10, Color::WHITE);
```

The font atlas should contain ASCII printable characters (32-126) arranged left-to-right, top-to-bottom.
