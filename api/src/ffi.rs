//! C-compatible FFI (Foreign Function Interface) for the pixel_zero API
//!
//! This module provides C-compatible functions and types for interacting with
//! the graphics and input APIs from C code.

use std::ptr;
use std::slice;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use crate::graphics::{GraphicsContext, color::Color, sprite::Sprite, font::BitmapFont};
use crate::input::{Input, Button};

// ============================================================================
// Graphics Context FFI
// ============================================================================

/// C-compatible opaque handle to GraphicsContext
pub type GraphicsContextHandle = *mut GraphicsContext;

/// Load and initialize the graphics context
/// Returns a handle to the graphics context, or null on failure
#[no_mangle]
pub extern "C" fn graphics_context_load() -> GraphicsContextHandle {
    match GraphicsContext::load() {
        Ok(ctx) => Box::into_raw(Box::new(ctx)),
        Err(_) => ptr::null_mut(),
    }
}

/// Free the graphics context
#[no_mangle]
pub extern "C" fn graphics_context_free(ctx: GraphicsContextHandle) {
    if !ctx.is_null() {
        unsafe {
            let _ = Box::from_raw(ctx);
        }
    }
}

/// Clear the framebuffer with the specified color
#[no_mangle]
pub extern "C" fn graphics_clear_framebuffer(ctx: GraphicsContextHandle, r: u8, g: u8, b: u8, a: u8) {
    if ctx.is_null() {
        return;
    }
    unsafe {
        (*ctx).clear_framebuffer(Color::rgba(r, g, b, a));
    }
}

/// Draw a sprite at the specified position
/// Returns 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn graphics_draw_sprite(
    ctx: GraphicsContextHandle,
    sprite: *const Sprite,
    x: c_int,
    y: c_int,
) -> c_int {
    if ctx.is_null() || sprite.is_null() {
        return -1;
    }
    unsafe {
        (*ctx).draw_sprite(&*sprite, x as i32, y as i32);
    }
    0
}

/// Draw text at the specified position with the given color
/// Returns 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn graphics_draw_text(
    ctx: GraphicsContextHandle,
    font: *const BitmapFont,
    text: *const c_char,
    x: c_int,
    y: c_int,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) -> c_int {
    if ctx.is_null() || font.is_null() || text.is_null() {
        return -1;
    }
    
    unsafe {
        let c_str = CStr::from_ptr(text);
        if let Ok(rust_str) = c_str.to_str() {
            (*ctx).draw_text(&*font, rust_str, x as i32, y as i32, Color::rgba(r, g, b, a));
            0
        } else {
            -1
        }
    }
}

/// Present the framebuffer to the screen
/// Returns 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn graphics_present(ctx: GraphicsContextHandle) -> c_int {
    if ctx.is_null() {
        return -1;
    }
    unsafe {
        match (*ctx).present() {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Get the framebuffer dimensions
/// Returns 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn graphics_framebuffer_size(
    ctx: GraphicsContextHandle,
    width: *mut u32,
    height: *mut u32,
) -> c_int {
    if ctx.is_null() || width.is_null() || height.is_null() {
        return -1;
    }
    unsafe {
        let (w, h) = (*ctx).framebuffer_size();
        *width = w;
        *height = h;
    }
    0
}

// ============================================================================
// Sprite FFI
// ============================================================================

/// C-compatible opaque handle to Sprite
pub type SpriteHandle = *mut Sprite;

/// Create a new sprite with the specified dimensions
/// Returns a handle to the sprite, or null on failure
#[no_mangle]
pub extern "C" fn sprite_new(width: u32, height: u32) -> SpriteHandle {
    Box::into_raw(Box::new(Sprite::new(width, height)))
}

/// Create a sprite from raw pixel data (RGBA format)
/// Returns a handle to the sprite, or null on failure
#[no_mangle]
pub extern "C" fn sprite_from_pixels(
    width: u32,
    height: u32,
    pixels: *const u8,
    pixels_len: usize,
) -> SpriteHandle {
    if pixels.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let pixel_slice = slice::from_raw_parts(pixels, pixels_len);
        let mut colors = Vec::with_capacity((width * height) as usize);
        
        for chunk in pixel_slice.chunks_exact(4) {
            colors.push(Color::rgba(chunk[0], chunk[1], chunk[2], chunk[3]));
        }
        
        match Sprite::from_pixels(width, height, colors) {
            Some(sprite) => Box::into_raw(Box::new(sprite)),
            None => ptr::null_mut(),
        }
    }
}

/// Free the sprite
#[no_mangle]
pub extern "C" fn sprite_free(sprite: SpriteHandle) {
    if !sprite.is_null() {
        unsafe {
            let _ = Box::from_raw(sprite);
        }
    }
}

/// Get the width of the sprite
#[no_mangle]
pub extern "C" fn sprite_width(sprite: *const Sprite) -> u32 {
    if sprite.is_null() {
        return 0;
    }
    unsafe { (*sprite).width() }
}

/// Get the height of the sprite
#[no_mangle]
pub extern "C" fn sprite_height(sprite: *const Sprite) -> u32 {
    if sprite.is_null() {
        return 0;
    }
    unsafe { (*sprite).height() }
}

/// Set a pixel in the sprite
/// Returns 0 on success, -1 on error
#[no_mangle]
pub extern "C" fn sprite_set_pixel(
    sprite: SpriteHandle,
    x: u32,
    y: u32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) -> c_int {
    if sprite.is_null() {
        return -1;
    }
    unsafe {
        (*sprite).set_pixel(x, y, Color::rgba(r, g, b, a));
    }
    0
}

/// Get a pixel from the sprite
/// Returns 0 on success, -1 if pixel is out of bounds
#[no_mangle]
pub extern "C" fn sprite_get_pixel(
    sprite: *const Sprite,
    x: u32,
    y: u32,
    r: *mut u8,
    g: *mut u8,
    b: *mut u8,
    a: *mut u8,
) -> c_int {
    if sprite.is_null() || r.is_null() || g.is_null() || b.is_null() || a.is_null() {
        return -1;
    }
    
    unsafe {
        match (*sprite).get_pixel(x, y) {
            Some(color) => {
                *r = color.r();
                *g = color.g();
                *b = color.b();
                *a = color.a();
                0
            }
            None => -1,
        }
    }
}

// ============================================================================
// BitmapFont FFI
// ============================================================================

/// C-compatible opaque handle to BitmapFont
pub type BitmapFontHandle = *mut BitmapFont;

/// Create a bitmap font from a sprite atlas
/// Returns a handle to the font, or null on failure
#[no_mangle]
pub extern "C" fn bitmap_font_new(
    atlas: *const Sprite,
    glyph_width: u32,
    glyph_height: u32,
) -> BitmapFontHandle {
    if atlas.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let sprite_clone = (*atlas).clone();
        Box::into_raw(Box::new(BitmapFont::new(sprite_clone, glyph_width, glyph_height)))
    }
}

/// Create a default bitmap font
/// Returns a handle to the font
#[no_mangle]
pub extern "C" fn bitmap_font_default() -> BitmapFontHandle {
    Box::into_raw(Box::new(BitmapFont::default()))
}

/// Free the bitmap font
#[no_mangle]
pub extern "C" fn bitmap_font_free(font: BitmapFontHandle) {
    if !font.is_null() {
        unsafe {
            let _ = Box::from_raw(font);
        }
    }
}

/// Get the glyph width of the font
#[no_mangle]
pub extern "C" fn bitmap_font_glyph_width(font: *const BitmapFont) -> u32 {
    if font.is_null() {
        return 0;
    }
    unsafe { (*font).glyph_width() }
}

/// Get the glyph height of the font
#[no_mangle]
pub extern "C" fn bitmap_font_glyph_height(font: *const BitmapFont) -> u32 {
    if font.is_null() {
        return 0;
    }
    unsafe { (*font).glyph_height() }
}

/// Measure the width of a text string in pixels
#[no_mangle]
pub extern "C" fn bitmap_font_measure_text(font: *const BitmapFont, text: *const c_char) -> u32 {
    if font.is_null() || text.is_null() {
        return 0;
    }
    
    unsafe {
        let c_str = CStr::from_ptr(text);
        if let Ok(rust_str) = c_str.to_str() {
            (*font).measure_text(rust_str)
        } else {
            0
        }
    }
}

// ============================================================================
// Input FFI
// ============================================================================

/// C-compatible opaque handle to Input
pub type InputHandle = *mut Input;

/// Create a new input handler
/// Returns a handle to the input handler
#[no_mangle]
pub extern "C" fn input_new() -> InputHandle {
    Box::into_raw(Box::new(Input::default()))
}

/// Free the input handler
#[no_mangle]
pub extern "C" fn input_free(input: InputHandle) {
    if !input.is_null() {
        unsafe {
            let _ = Box::from_raw(input);
        }
    }
}

/// Update the input state (call once per frame)
#[no_mangle]
pub extern "C" fn input_update(input: InputHandle) {
    if input.is_null() {
        return;
    }
    unsafe {
        (*input).update();
    }
}

/// C-compatible button enumeration
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CButton {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    A = 4,
    B = 5,
    L = 6,
    R = 7,
    Start = 8,
    Select = 9,
}

impl From<CButton> for Button {
    fn from(btn: CButton) -> Self {
        match btn {
            CButton::Up => Button::Up,
            CButton::Down => Button::Down,
            CButton::Left => Button::Left,
            CButton::Right => Button::Right,
            CButton::A => Button::A,
            CButton::B => Button::B,
            CButton::L => Button::L,
            CButton::R => Button::R,
            CButton::Start => Button::Start,
            CButton::Select => Button::Select,
        }
    }
}

/// Check if a button is currently pressed
/// Returns 1 if pressed, 0 if not
#[no_mangle]
pub extern "C" fn input_is_pressed(input: *const Input, button: CButton) -> c_int {
    if input.is_null() {
        return 0;
    }
    unsafe {
        if (*input).is_pressed(button.into()) {
            1
        } else {
            0
        }
    }
}

/// Check if a button was just pressed this frame
/// Returns 1 if just pressed, 0 if not
#[no_mangle]
pub extern "C" fn input_just_pressed(input: *const Input, button: CButton) -> c_int {
    if input.is_null() {
        return 0;
    }
    unsafe {
        if (*input).just_pressed(button.into()) {
            1
        } else {
            0
        }
    }
}

/// Check if a button was just released this frame
/// Returns 1 if just released, 0 if not
#[no_mangle]
pub extern "C" fn input_just_released(input: *const Input, button: CButton) -> c_int {
    if input.is_null() {
        return 0;
    }
    unsafe {
        if (*input).just_released(button.into()) {
            1
        } else {
            0
        }
    }
}
