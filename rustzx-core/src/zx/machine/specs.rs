use crate::zx::constants::BORDER_COLS;
use alloc::{ vec::Vec, vec };

/// Immutable type (Builder is not public in outer module)
pub struct ZXSpecs {
    // frequencies
    pub freq_cpu: usize,
    // first_pixel_read_clocks (contention start)
    pub clocks_first_pixel: usize,
    // row clocks
    pub clocks_left_border: usize,
    pub clocks_screen_row: usize,
    pub clocks_right_border: usize,
    pub clocks_retrace: usize,
    pub clocks_line: usize,
    pub clocks_line_base: Vec<usize>,
    // some ula clocks
    pub clocks_ula_read_shift: usize,
    pub clocks_ula_read_origin: usize,
    pub clocks_ula_contention_origin: usize,
    pub clocks_ula_beam_shift: usize,
    // frame
    pub clocks_frame: usize,
    // lines metrics
    pub lines_vsync: usize,
    pub lines_top_border: usize,
    pub lines_screen: usize,
    pub lines_bottom_border: usize,
    pub lines_all: usize,
    // interrupt
    pub interrupt_length: usize,
    // contention
    pub contention_offset: usize,
    pub contention_pattern: [usize; 8],
}

/// Specs builder, used to make static valiables with machines specs
pub struct ZXSpecsBuilder {
    specs: ZXSpecs,
}
impl ZXSpecsBuilder {
    /// Returns new ZXSpecsBuilder
    pub fn new() -> ZXSpecsBuilder {
        ZXSpecsBuilder {
            specs: ZXSpecs {
                // frequencies
                freq_cpu: 0,
                // first_pixel_clocks
                clocks_first_pixel: 0,
                // row clocks
                clocks_left_border: 0,
                clocks_screen_row: 0,
                clocks_right_border: 0,
                clocks_retrace: 0,
                clocks_line: 0,
                clocks_line_base: vec![],
                // some ula clocks
                clocks_ula_read_shift: 0,
                clocks_ula_read_origin: 0,
                clocks_ula_contention_origin: 0,
                clocks_ula_beam_shift: 0,
                // frame clocks
                clocks_frame: 0,
                // lines metrics
                lines_vsync: 0,
                lines_top_border: 0,
                lines_screen: 0,
                lines_bottom_border: 0,
                lines_all: 0,
                // interrupt
                interrupt_length: 0,
                // contention
                contention_offset: 0,
                contention_pattern: [0; 8],
            },
        }
    }

    /// Builds new ZXSpecs
    pub fn build(mut self) -> ZXSpecs {
        self.specs.clocks_frame =
            (self.specs.lines_all + self.specs.lines_vsync) * self.specs.clocks_line;
        // 4*4 is 4 border columns * 4 clocks per column
        self.specs.clocks_line_base.push(
            self.specs.clocks_first_pixel
                - self.specs.lines_top_border * self.specs.clocks_line
                - BORDER_COLS * 4,
        );
        // + 1 because TStates in calculations may be > frame length (CHECK)
        let lines_count = self.specs.lines_all + 1;
        for _ in 1..lines_count {
            let last = *self.specs.clocks_line_base.last().unwrap();
            let line_clocks = self.specs.clocks_line;
            self.specs.clocks_line_base.push(last + line_clocks);
        }
        self.specs.clocks_ula_read_origin =
            self.specs.clocks_first_pixel + self.specs.clocks_ula_read_shift;
        self.specs.clocks_ula_contention_origin =
            self.specs.clocks_first_pixel - self.specs.contention_offset;
        self.specs
    }

    /// Changes CPU frequency
    pub fn freq_cpu(mut self, value: usize) -> Self {
        self.specs.freq_cpu = value;
        self
    }

    /// Changes Clocks per left border, screen render, left border and retrace
    pub fn clocks_row(
        mut self,
        lborder: usize,
        screen: usize,
        rborder: usize,
        retrace: usize,
    ) -> Self {
        self.specs.clocks_left_border = lborder;
        self.specs.clocks_screen_row = screen;
        self.specs.clocks_right_border = rborder;
        self.specs.clocks_retrace = retrace;
        self.specs.clocks_line = lborder + screen + rborder + retrace;
        self
    }

    /// Changes first pixel clocks
    pub fn clocks_first_pixel(mut self, value: usize) -> Self {
        self.specs.clocks_first_pixel = value;
        self
    }

    /// Changes shift of time, when ula reads data from memory
    pub fn clocks_ula_read_shift(mut self, value: usize) -> Self {
        self.specs.clocks_ula_read_shift = value;
        self
    }

    /// Changes shift of electron beam pixel rendering
    pub fn clocks_ula_beam_shift(mut self, value: usize) -> Self {
        self.specs.clocks_ula_beam_shift = value;
        self
    }

    /// Changes lines per top border, screen, bottom border and vsync
    pub fn lines(mut self, tborder: usize, screen: usize, bborder: usize, vsync: usize) -> Self {
        self.specs.lines_vsync = vsync;
        self.specs.lines_top_border = tborder;
        self.specs.lines_screen = screen;
        self.specs.lines_bottom_border = bborder;
        self.specs.lines_all = tborder + screen + bborder;
        self
    }

    /// Changes contention pattern
    pub fn contention(mut self, pattern: [usize; 8], offset: usize) -> Self {
        self.specs.contention_pattern = pattern;
        self.specs.contention_offset = offset;
        self
    }

    /// changes interrupt length
    pub fn interrupt_length(mut self, value: usize) -> Self {
        self.specs.interrupt_length = value;
        self
    }
}
