// Simple 8x8 bitmap font data
// Each character is 8x8 pixels, stored as bytes (0=empty, 1=filled)
// Format: [char_code][row0][row1]...[row7]

pub const FONT_WIDTH: u32 = 8;
pub const FONT_HEIGHT: u32 = 8;

// Font data is generated procedurally in renderer.rs::create_bitmap_font()
