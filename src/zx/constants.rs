//! Module contains constants
//! addresses
pub const BITMAP_BASE_ADDR: u16 = 0x4000;
pub const ATTR_BASE_ADDR: u16 = 0x5800;
pub const ATTR_MAX_ADDR: u16 = 0x5AFF;
// relative addresses
pub const BITMAP_MAX_REL: u16 = 0x17FF;
pub const ATTR_BASE_REL: u16 = 0x1800;
pub const ATTR_MAX_REL: u16 = 0x1AFF;

// main spectrum screen (canvas) constants
pub const CANVAS_WIDTH: usize = 256;
pub const CANVAS_HEIGHT: usize = 192;
pub const CANVAS_X: usize = 32;
pub const CANVAS_Y: usize = 24;
// on all spectrums theese values are fixed
pub const CLOCKS_PER_COL: usize = 4;
pub const PIXELS_PER_CLOCK: usize = 2;
// size of screen in rows, cols
pub const ATTR_COLS: usize = CANVAS_WIDTH / 8;
pub const ATTR_ROWS: usize = CANVAS_HEIGHT / 8;
pub const BORDER_COLS: usize = 4;
pub const BORDER_ROWS: usize = 3;
// canvas (emulated screen) constants
pub const SCREEN_WIDTH: usize = CANVAS_WIDTH + BORDER_COLS * 8 * 2;
pub const SCREEN_HEIGHT: usize = CANVAS_HEIGHT + BORDER_ROWS * 8 * 2;
pub const PIXEL_COUNT: usize = SCREEN_HEIGHT * SCREEN_WIDTH;
pub const BYTES_PER_PIXEL: usize = 4;
// 44100 Hz Sample rate
pub const SAMPLE_RATE: usize = 44100;
// Frames per second
pub const FPS: usize = 50;
