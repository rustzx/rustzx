//! Module describes ZX Spectrum screen
//! *block* - is 8x1 pxels stripe.
use utils::screen::*;
use utils::*;
use zx::constants::*;
use zx::machine::ZXMachine;
use zx::screen::colors::*;

// size of screen buffer in bytes
const BUFFER_LENGTH: usize = CANVAS_HEIGHT * CANVAS_WIDTH * BYTES_PER_PIXEL;

/// Represents how much 8x1 have been **passed**.
#[derive(PartialEq, Eq, Debug)]
pub struct BlocksCount {
    pub lines: usize,
    pub columns: usize,
}

impl BlocksCount {
    /// Constructs new `BlocksCount`
    pub fn new(lines: usize, columns: usize) -> BlocksCount {
        BlocksCount {
            lines: lines,
            columns: columns,
        }
    }

    /// Constructs self from clocks count, taking into account machine type
    pub fn from_clocks(clocks: Clocks, machine: ZXMachine) -> BlocksCount {
        // get reference to specs for less words
        let specs = machine.specs();
        let mut lines;
        let mut columns;
        if clocks.count() < specs.clocks_ula_read_origin {
            // zero blocks passed
            lines = 0;
            columns = 0;
        } else {
            // clocks relative to first pixel rendering
            let clocks = clocks.count() - specs.clocks_ula_read_origin;
            // so find passed lines and columns count
            lines = clocks / specs.clocks_line;
            columns = (clocks % specs.clocks_line) / CLOCKS_PER_COL;
            // columns must contain PASSED blocks, so increment it.
            columns += 1;
            // if out of visible canvas line
            if columns > ATTR_COLS {
                lines += 1;
                columns = 0;
            };
            if lines >= CANVAS_HEIGHT {
                lines = 0;
                columns = 0;
            }
        };
        BlocksCount {
            lines: lines,
            columns: columns,
        }
    }

    /// Returns count of blocks between positions
    /// # Notes
    /// `prev` must be lover than `self`
    pub fn passed_from(&self, prev: &BlocksCount) -> usize {
        if self.lines < prev.lines {
            ATTR_COLS - prev.columns
        //self.lines * ATTR_COLS + self.columns
        } else if self.lines == prev.lines {
            // if positions on the same line => just use difference in columns.
            self.columns - prev.columns
        } else {
            // add blocks left from start line, blocks on lines between and blocks on end line
            (ATTR_COLS - prev.columns) + (self.lines - prev.lines - 1) * ATTR_COLS + self.columns
        }
    }
}

/// Represents Single memory bank of screen
struct ScreenBank {
    pub attributes: Box<[ZXAttribute; ATTR_COLS * ATTR_ROWS]>,
    pub bitmap: Box<[u8; ATTR_COLS * CANVAS_HEIGHT]>,
}

/// Represents ZXSpectrum emulated mid part of screen (canvas)
pub struct ZXCanvas {
    machine: ZXMachine,
    palette: ZXPalette,
    last_blocks: BlocksCount,
    flash: bool,
    frame_counter: usize,
    // bitmap buffers
    buffer: Box<[u8; BUFFER_LENGTH]>,
    back_buffer: Box<[u8; BUFFER_LENGTH]>,
    // memory
    banks: [ScreenBank; 2],
    active_bank: usize,
    next_bank: usize,
}

impl ZXCanvas {
    /// Constructs new canvas of `machine`
    pub fn new(machine: ZXMachine) -> ZXCanvas {
        ZXCanvas {
            machine: machine,
            palette: ZXPalette::default(),
            last_blocks: BlocksCount::new(0, 0),
            flash: false,
            frame_counter: 0,
            buffer: Box::new([0; BUFFER_LENGTH]),
            back_buffer: Box::new([0; BUFFER_LENGTH]),
            banks: [
                ScreenBank {
                    attributes: Box::new([ZXAttribute::from_byte(0); ATTR_COLS * ATTR_ROWS]),
                    bitmap: Box::new([0; ATTR_COLS * CANVAS_HEIGHT]),
                },
                ScreenBank {
                    attributes: Box::new([ZXAttribute::from_byte(0); ATTR_COLS * ATTR_ROWS]),
                    bitmap: Box::new([0; ATTR_COLS * CANVAS_HEIGHT]),
                },
            ],
            active_bank: 0,
            next_bank: 0,
        }
    }

    /// changes flash switch
    fn switch_flash(&mut self) {
        self.flash = !self.flash;
    }

    /// transforms zx spectrum bank to local index
    fn local_bank(&self, bank: usize) -> Option<usize> {
        match self.machine {
            ZXMachine::Sinclair48K if bank == 0 => Some(0),
            ZXMachine::Sinclair128K if bank == 5 => Some(0),
            ZXMachine::Sinclair128K if bank == 7 => Some(1),
            _ => None,
        }
    }

    /// selects bank of memory
    pub fn switch_bank(&mut self, bank: usize) {
        if let Some(bank) = self.local_bank(bank) {
            self.active_bank = bank;
        }
    }

    /// renders some  8x1 blocks
    /// `clocks` - current  clocks count form frame start.
    /// if clocks < previous call clocks then discard processing
    pub fn process_clocks(&mut self, clocks: Clocks) {
        let blocks = BlocksCount::from_clocks(clocks, self.machine);
        // so, let's count of 8x1 blocks, which passed.
        let count = blocks.passed_from(&self.last_blocks);
        if count > 0 {
            // fill pixels from prev to current
            let prev_block = self.last_blocks.lines * ATTR_COLS + self.last_blocks.columns;
            let curr_block = blocks.lines * ATTR_COLS + blocks.columns;
            // so we know that some blocks have been passed
            // block holds current blocks index
            for block in prev_block..curr_block {
                let bitmap = self.banks[self.active_bank].bitmap[block];
                // one attr per 8x8 area
                let attr_row = block / (ATTR_COLS * 8);
                let attr_col = block % ATTR_COLS;
                let attr = self.banks[self.active_bank].attributes[attr_row * ATTR_COLS + attr_col];
                for pixel in 0..8 {
                    // from most significant bit
                    let state = ((bitmap << pixel) & 0x80) != 0;
                    let color = self
                        .palette
                        .get_rgba(attr.active_color(state, self.flash), attr.brightness);
                    let index = (block * 8 + pixel) * BYTES_PER_PIXEL;
                    self.buffer[index..index + BYTES_PER_PIXEL].clone_from_slice(color);
                }
            }
            // cahnge last block to current
            self.last_blocks = blocks;
        }
    }

    /// starts new frame
    pub fn new_frame(&mut self) {
        // post finished bitmap to second buffer (all not-rendered part will be updated)
        self.back_buffer.clone_from_slice(&(*self.buffer));
        self.last_blocks = BlocksCount::new(0, 0);
        if self.frame_counter % 16 == 0 {
            self.switch_flash();
        }
        self.frame_counter += 1;
    }

    /// Updates data if screen ram
    pub fn update(&mut self, rel_addr: u16, bank: usize, data: u8) {
        if let Some(bank) = self.local_bank(bank) {
            match rel_addr {
                // change bitmap
                0...BITMAP_MAX_REL => {
                    let line = bitmap_line_rel(rel_addr);
                    let col = bitmap_col_rel(rel_addr);
                    self.banks[bank].bitmap[line * ATTR_COLS + col] = data;
                }
                // change attribute
                ATTR_BASE_REL...ATTR_MAX_REL => {
                    let row = attr_row_rel(rel_addr);
                    let col = attr_col_rel(rel_addr);
                    self.banks[bank].attributes[row * ATTR_COLS + col] =
                        ZXAttribute::from_byte(data);
                }
                // no screen changes
                _ => {}
            }
        }
    }

    /// Returns reference to canvas main texture
    pub fn texture(&self) -> &[u8] {
        &(*self.buffer)
    }
}
