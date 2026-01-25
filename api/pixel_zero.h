/**
 * Pixel Zero C API
 * 
 * C-compatible API for the pixel_zero graphics and input library.
 * Provides GBA-style 2D sprite rendering with a fixed 320x240 resolution.
 */

#ifndef PIXEL_ZERO_H
#define PIXEL_ZERO_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// Graphics Context API
// ============================================================================

/** Opaque handle to a graphics context */
typedef struct GraphicsContext* GraphicsContextHandle;

/**
 * Load and initialize the graphics context.
 * @return Handle to the graphics context, or NULL on failure
 */
GraphicsContextHandle graphics_context_load(void);

/**
 * Free the graphics context and release resources.
 * @param ctx Graphics context handle
 */
void graphics_context_free(GraphicsContextHandle ctx);

/**
 * Clear the framebuffer with the specified RGBA color.
 * @param ctx Graphics context handle
 * @param r Red component (0-255)
 * @param g Green component (0-255)
 * @param b Blue component (0-255)
 * @param a Alpha component (0-255)
 */
void graphics_clear_framebuffer(GraphicsContextHandle ctx, uint8_t r, uint8_t g, uint8_t b, uint8_t a);

/**
 * Draw a sprite at the specified position.
 * @param ctx Graphics context handle
 * @param sprite Sprite to draw
 * @param x X position (pixels from left)
 * @param y Y position (pixels from top)
 * @return 0 on success, -1 on error
 */
int graphics_draw_sprite(GraphicsContextHandle ctx, const void* sprite, int x, int y);

/**
 * Draw text at the specified position with the given color.
 * @param ctx Graphics context handle
 * @param font Font to use
 * @param text Null-terminated string to draw
 * @param x X position (pixels from left)
 * @param y Y position (pixels from top)
 * @param r Red component (0-255)
 * @param g Green component (0-255)
 * @param b Blue component (0-255)
 * @param a Alpha component (0-255)
 * @return 0 on success, -1 on error
 */
int graphics_draw_text(
    GraphicsContextHandle ctx,
    const void* font,
    const char* text,
    int x, int y,
    uint8_t r, uint8_t g, uint8_t b, uint8_t a
);

/**
 * Present the framebuffer to the screen with automatic letterboxing.
 * @param ctx Graphics context handle
 * @return 0 on success, -1 on error
 */
int graphics_present(GraphicsContextHandle ctx);

/**
 * Get the dimensions of the framebuffer.
 * @param ctx Graphics context handle
 * @param width Pointer to store width (320)
 * @param height Pointer to store height (240)
 * @return 0 on success, -1 on error
 */
int graphics_framebuffer_size(GraphicsContextHandle ctx, uint32_t* width, uint32_t* height);

// ============================================================================
// Sprite API
// ============================================================================

/** Opaque handle to a sprite */
typedef struct Sprite* SpriteHandle;

/**
 * Create a new sprite with the specified dimensions.
 * All pixels are initialized to transparent black (RGBA: 0,0,0,0).
 * @param width Width in pixels
 * @param height Height in pixels
 * @return Handle to the sprite, or NULL on failure
 */
SpriteHandle sprite_new(uint32_t width, uint32_t height);

/**
 * Create a sprite from raw RGBA pixel data.
 * @param width Width in pixels
 * @param height Height in pixels
 * @param pixels Pointer to RGBA pixel data (4 bytes per pixel)
 * @param pixels_len Length of pixel data in bytes (must be width * height * 4)
 * @return Handle to the sprite, or NULL on failure
 */
SpriteHandle sprite_from_pixels(uint32_t width, uint32_t height, const uint8_t* pixels, size_t pixels_len);

/**
 * Free the sprite and release resources.
 * @param sprite Sprite handle
 */
void sprite_free(SpriteHandle sprite);

/**
 * Get the width of the sprite.
 * @param sprite Sprite handle
 * @return Width in pixels, or 0 if sprite is NULL
 */
uint32_t sprite_width(const void* sprite);

/**
 * Get the height of the sprite.
 * @param sprite Sprite handle
 * @return Height in pixels, or 0 if sprite is NULL
 */
uint32_t sprite_height(const void* sprite);

/**
 * Set a pixel in the sprite to the specified RGBA color.
 * @param sprite Sprite handle
 * @param x X coordinate
 * @param y Y coordinate
 * @param r Red component (0-255)
 * @param g Green component (0-255)
 * @param b Blue component (0-255)
 * @param a Alpha component (0-255)
 * @return 0 on success, -1 on error
 */
int sprite_set_pixel(SpriteHandle sprite, uint32_t x, uint32_t y, uint8_t r, uint8_t g, uint8_t b, uint8_t a);

/**
 * Get a pixel from the sprite.
 * @param sprite Sprite handle
 * @param x X coordinate
 * @param y Y coordinate
 * @param r Pointer to store red component
 * @param g Pointer to store green component
 * @param b Pointer to store blue component
 * @param a Pointer to store alpha component
 * @return 0 on success, -1 if pixel is out of bounds
 */
int sprite_get_pixel(const void* sprite, uint32_t x, uint32_t y, uint8_t* r, uint8_t* g, uint8_t* b, uint8_t* a);

// ============================================================================
// BitmapFont API
// ============================================================================

/** Opaque handle to a bitmap font */
typedef struct BitmapFont* BitmapFontHandle;

/**
 * Create a bitmap font from a sprite atlas.
 * The atlas should contain ASCII printable characters (32-126) arranged left-to-right, top-to-bottom.
 * @param atlas Sprite containing the font atlas
 * @param glyph_width Width of each glyph in pixels
 * @param glyph_height Height of each glyph in pixels
 * @return Handle to the font, or NULL on failure
 */
BitmapFontHandle bitmap_font_new(const void* atlas, uint32_t glyph_width, uint32_t glyph_height);

/**
 * Create a default 8x8 bitmap font.
 * @return Handle to the font
 */
BitmapFontHandle bitmap_font_default(void);

/**
 * Free the bitmap font and release resources.
 * @param font Font handle
 */
void bitmap_font_free(BitmapFontHandle font);

/**
 * Get the glyph width of the font.
 * @param font Font handle
 * @return Glyph width in pixels, or 0 if font is NULL
 */
uint32_t bitmap_font_glyph_width(const void* font);

/**
 * Get the glyph height of the font.
 * @param font Font handle
 * @return Glyph height in pixels, or 0 if font is NULL
 */
uint32_t bitmap_font_glyph_height(const void* font);

/**
 * Measure the width of a text string in pixels.
 * @param font Font handle
 * @param text Null-terminated string to measure
 * @return Width in pixels, or 0 if font or text is NULL
 */
uint32_t bitmap_font_measure_text(const void* font, const char* text);

// ============================================================================
// Input API
// ============================================================================

/** Opaque handle to an input handler */
typedef struct Input* InputHandle;

/** Button enumeration */
typedef enum {
    BUTTON_UP = 0,
    BUTTON_DOWN = 1,
    BUTTON_LEFT = 2,
    BUTTON_RIGHT = 3,
    BUTTON_A = 4,
    BUTTON_B = 5,
    BUTTON_L = 6,
    BUTTON_R = 7,
    BUTTON_START = 8,
    BUTTON_SELECT = 9
} CButton;

/**
 * Create a new input handler.
 * @return Handle to the input handler
 */
InputHandle input_new(void);

/**
 * Free the input handler and release resources.
 * @param input Input handle
 */
void input_free(InputHandle input);

/**
 * Update the input state. Call this once per frame before checking button states.
 * @param input Input handle
 */
void input_update(InputHandle input);

/**
 * Check if a button is currently pressed.
 * @param input Input handle
 * @param button Button to check
 * @return 1 if pressed, 0 if not
 */
int input_is_pressed(const void* input, CButton button);

/**
 * Check if a button was just pressed this frame.
 * @param input Input handle
 * @param button Button to check
 * @return 1 if just pressed, 0 if not
 */
int input_just_pressed(const void* input, CButton button);

/**
 * Check if a button was just released this frame.
 * @param input Input handle
 * @param button Button to check
 * @return 1 if just released, 0 if not
 */
int input_just_released(const void* input, CButton button);

#ifdef __cplusplus
}
#endif

#endif /* PIXEL_ZERO_H */
