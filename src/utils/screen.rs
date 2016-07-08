use zx::constants::*;
use super::split_word;


/// Encode line number to read memory address
pub fn bitmap_line_addr(line: usize) -> u16 {
    assert!(line < CANVAS_HEIGHT);
    // 0 1 0 Y7 Y6 Y2 Y1 Y0 | Y5 Y4 Y3 X4 X3 X2 X1 X0
    (0x4000 | (line << 5) & 0x1800 | (line << 8) & 0x0700 | (line << 2) & 0x00E0) as u16
}

/// Get pixel id from address
pub fn bitmap_line_rel(addr: u16) -> usize {
    assert!(addr < ATTR_BASE_REL);
    let (h, l) = split_word(addr);
    // 0 0 0 Y7 Y6 Y2 Y1 Y0 | Y5 Y4 Y3 X4 X3 X2 X1 X0
    // extract lowest 5 bits as x coordinate base
    let y = (h & 0x07) | ((l >> 2) & 0x38) | ((h << 3) & 0xC0);
    y as usize
}

/// get bitmap column from address
pub fn bitmap_col_rel(addr: u16) -> usize {
    assert!(addr < ATTR_BASE_REL);
    let (_, l) = split_word(addr);
    // extract lowest 5 bits as x coordinate base
    (l & 0x1F) as usize
}

/// get attribute row from address
pub fn attr_row_rel(addr: u16) -> usize {
    assert!(addr >= ATTR_BASE_REL && addr <= ATTR_MAX_REL);
    ((addr - ATTR_BASE_REL) / ATTR_COLS as u16) as usize
}

/// get attribute column from address
pub fn attr_col_rel(addr: u16) -> usize {
    assert!(addr >= ATTR_BASE_REL && addr <= ATTR_MAX_REL);
    ((addr - ATTR_BASE_REL) % ATTR_COLS as u16) as usize
}
