//! Module contains constants

/// main spectrum screen (canvas) constants
pub const CANVAS_WIDTH: usize = 256;
pub const CANVAS_HEIGHT: usize = 192;
pub const CANVAS_X: usize = 32;
pub const CANVAS_Y: usize = 24;
/// canvas (emulated screen) constants
pub const SCREEN_WIDTH: usize = CANVAS_WIDTH + BORDER_COLS * 8 * 2;
pub const SCREEN_HEIGHT: usize = CANVAS_HEIGHT + BORDER_ROWS * 8 * 2;
/// Frames per second
pub const FPS: usize = 50;

/// relative addresses
pub(crate) const BITMAP_MAX_REL: u16 = 0x17FF;
pub(crate) const ATTR_BASE_REL: u16 = 0x1800;
pub(crate) const ATTR_MAX_REL: u16 = 0x1AFF;
/// on all spectrums theese values are fixed
pub(crate) const CLOCKS_PER_COL: usize = 4;
#[cfg(feature = "precise-border")]
pub(crate) const PIXELS_PER_CLOCK: usize = 2;
/// size of screen in rows, cols
pub(crate) const ATTR_COLS: usize = CANVAS_WIDTH / 8;
pub(crate) const ATTR_ROWS: usize = CANVAS_HEIGHT / 8;
pub(crate) const BORDER_COLS: usize = 4;
pub(crate) const BORDER_ROWS: usize = 3;
/// Tape loading trap at LD-BREAK routine in ROM
pub(crate) const ADDR_LD_BREAK: u16 = 0x056B;
