//! Module with machine specifications

// Allow outer modules to use ZXSpecs struct, but not construct
mod specs;
use self::specs::ZXSpecsBuilder;
use utils::Clocks;
pub use self::specs::ZXSpecs;

lazy_static! {
    /// ZX Spectrum 48K Specs
    pub static ref SPECS_48K: ZXSpecs = {
        ZXSpecsBuilder::new()
            .freq_cpu(3_500_000)
            .clocks_first_pixel(14336)
            .clocks_ula_read_shift(2)
            .clocks_ula_beam_shift(1)
            .clocks_row(24, 128, 24, 48)
            .lines(48, 192, 48, 24)
            .contention([6, 5, 4, 3, 2, 1, 0, 0], 1)
            .interrupt_length(32)
            .build()
        };
}

lazy_static! {
    /// ZX Spectrum 128K Specs
    pub static ref SPECS_128K: ZXSpecs = {
        ZXSpecsBuilder::new()
            .freq_cpu(3_546_900)
            .clocks_first_pixel(14362)
            .clocks_ula_read_shift(2)
            .clocks_ula_beam_shift(1)
            .clocks_row(24, 128, 24, 52)
            .lines(48, 192, 48, 23)
            .contention([6, 5, 4, 3, 2, 1, 0, 0], 1)
            .interrupt_length(32)
            .build()
    };
}

/// Machine type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZXMachine {
    Sinclair48K,
    Sinclair128K,
}

impl ZXMachine {
    /// Returns current machine specs as ref to static value
    pub fn specs(self) -> &'static ZXSpecs {
        match self {
            ZXMachine::Sinclair48K => &SPECS_48K,
            ZXMachine::Sinclair128K => &SPECS_128K,
        }
    }

    /// Returns contention during specified time
    pub fn contention_clocks(self, clocks: Clocks) -> Clocks {
        let specs = self.specs();
        if (clocks.count() < (specs.clocks_first_pixel - 1)) ||
           (clocks.count() >=
            (specs.clocks_first_pixel - 1) + specs.lines_screen * specs.clocks_line) {
            return Clocks(0);
        }
        let clocks_trough_line = (clocks.count() - (specs.clocks_first_pixel - 1)) %
                                 specs.clocks_line;
        if clocks_trough_line >= specs.clocks_screen_row {
            return Clocks(0);
        }
        return Clocks(self.specs().contention_pattern[(clocks_trough_line % 8) as usize]);
    }

    /// Checks port contention on machine
    pub fn port_is_contended(self, port: u16) -> bool {
        match self {
            ZXMachine::Sinclair48K | ZXMachine::Sinclair128K => {
                // every even port
                (port & 0x0001) == 0
            }
        }
    }

    /// Returns contention status of bank
    pub fn bank_is_contended(self, page: usize) -> bool {
        match self {
            ZXMachine::Sinclair48K => {
                page == 0
            }
            ZXMachine::Sinclair128K => {
                let contended_pages = [1, 3, 5, 7];
                contended_pages.iter().any(|&x| x == page)
            }
        }
    }
}
