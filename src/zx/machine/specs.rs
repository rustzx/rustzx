use zx::screen::BORDER_WIDTH_COLUMNS;

pub struct ZXSpecs {
    // frequencies
    pub freq_cpu: u64,
    // first_pixel_clocks
    pub clocks_first_pixel: u64,
    // row clocks
    pub clocks_left_border: u64,
    pub clocks_screen_row: u64,
    pub clocks_right_border: u64,
    pub clocks_retrace: u64,
    pub clocks_line: u64,
    pub clocks_line_base: Vec<u64>,
    // frame
    pub clocks_frame: u64,
    // lines metrics
    pub lines_vsync: u64,
    pub lines_top_border: u64,
    pub lines_screen: u64,
    pub lines_bottom_border: u64,
    pub lines_all: u64,
    // interrupt
    pub interrupt_length: u64,
    // contention
    pub contention_offset: u64,
    pub contention_pattern: [u64; 8],
}

pub struct ZXSpecsBuilder {
    specs: ZXSpecs,
}
impl ZXSpecsBuilder {
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
                // frame clocks
                // TODO: eval
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
            }
        }
    }
    pub fn build(mut self) -> ZXSpecs {
        self.specs.clocks_frame = (self.specs.lines_all + self.specs.lines_vsync) *
            self.specs.clocks_line;
        // 4*4 is 4 border columns * 4 clocks per column
        self.specs.clocks_line_base.push(self.specs.clocks_first_pixel -
            self.specs.lines_top_border * self.specs.clocks_line -
            BORDER_WIDTH_COLUMNS as u64 * 4);
        // + 1 because TStates in calculations may be > frame length (CHECK)
        let lines_count = self.specs.lines_all + 1;
        for _ in 1..lines_count {
            let last = *self.specs.clocks_line_base.last().unwrap();
            let line_clocks = self.specs.clocks_line;
            self.specs.clocks_line_base.push(last + line_clocks);
        };
        self.specs
    }
    pub fn freq_cpu(mut self, value: u64) -> Self {
        self.specs.freq_cpu = value;
        self
    }
    pub fn clocks_row(mut self, lborder: u64, screen: u64, rborder: u64, retrace: u64) -> Self {
        self.specs.clocks_left_border = lborder;
        self.specs.clocks_screen_row = screen;
        self.specs.clocks_right_border = rborder;
        self.specs.clocks_retrace = retrace;
        self.specs.clocks_line = lborder + screen + rborder + retrace;
        self
    }
    pub fn clocks_first_pixel(mut self, value: u64) -> Self {
        self.specs.clocks_first_pixel = value;
        self
    }
    pub fn lines(mut self, tborder: u64, screen: u64, bborder: u64, vsync: u64) -> Self {
        self.specs.lines_vsync = vsync;
        self.specs.lines_top_border = tborder;
        self.specs.lines_screen = screen;
        self.specs.lines_bottom_border = bborder;
        self.specs.lines_all = tborder + screen + bborder;
        self
    }
    pub fn contention(mut self, pattern: [u64; 8], offset: u64) -> Self {
        self.specs.contention_pattern = pattern;
        self.specs.contention_offset = offset;
        self
    }
    pub fn interrupt_length(mut self, value: u64) -> Self {
        self.specs.interrupt_length = value;
        self
    }
}
