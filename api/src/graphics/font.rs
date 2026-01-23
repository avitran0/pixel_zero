use crate::graphics::{color::Color, sprite::Sprite};

/// A bitmap font for rendering text
#[derive(Debug, Clone)]
pub struct BitmapFont {
    glyph_width: u32,
    glyph_height: u32,
    /// Font atlas containing all glyphs in a single sprite
    atlas: Sprite,
    /// Characters per row in the atlas
    chars_per_row: u32,
}

impl BitmapFont {
    /// Create a new bitmap font from an atlas sprite
    /// The atlas should contain glyphs arranged in a grid
    pub fn new(atlas: Sprite, glyph_width: u32, glyph_height: u32) -> Self {
        let chars_per_row = atlas.width() / glyph_width;
        Self {
            glyph_width,
            glyph_height,
            atlas,
            chars_per_row,
        }
    }

    /// Get the width of each glyph
    pub fn glyph_width(&self) -> u32 {
        self.glyph_width
    }

    /// Get the height of each glyph
    pub fn glyph_height(&self) -> u32 {
        self.glyph_height
    }

    /// Get a sprite for a specific character
    /// Assumes ASCII ordering starting from space (32)
    pub fn get_glyph(&self, character: char) -> Option<Sprite> {
        let char_code = character as u32;
        // Support ASCII printable characters (32-126)
        if char_code < 32 || char_code > 126 {
            return None;
        }

        let glyph_index = char_code - 32;
        let glyph_x = (glyph_index % self.chars_per_row) * self.glyph_width;
        let glyph_y = (glyph_index / self.chars_per_row) * self.glyph_height;

        let mut glyph = Sprite::new(self.glyph_width, self.glyph_height);
        for y in 0..self.glyph_height {
            for x in 0..self.glyph_width {
                if let Some(color) = self.atlas.get_pixel(glyph_x + x, glyph_y + y) {
                    glyph.set_pixel(x, y, color);
                }
            }
        }
        Some(glyph)
    }

    /// Measure the width of a text string in pixels
    pub fn measure_text(&self, text: &str) -> u32 {
        text.len() as u32 * self.glyph_width
    }
}

impl Default for BitmapFont {
    /// Create a simple default 8x8 font
    fn default() -> Self {
        // Create a simple monospace font with basic ASCII characters
        // This is a minimal implementation - users should provide their own fonts
        let glyph_width = 8;
        let glyph_height = 8;
        let chars_per_row = 16;
        let rows = 6; // 95 printable ASCII chars / 16 = 6 rows

        let atlas = Sprite::new(chars_per_row * glyph_width, rows * glyph_height);
        
        Self {
            glyph_width,
            glyph_height,
            atlas,
            chars_per_row,
        }
    }
}
