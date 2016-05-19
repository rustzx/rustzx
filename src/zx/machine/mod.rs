//! Module with machine specifications

// Allow outer modules to use ZXSpecs struct, but not construct
mod specs;
use self::specs::ZXSpecsBuilder;
pub use self::specs::ZXSpecs;

lazy_static! {
    /// ZX Spectrum 48K Specs
    pub static ref SPECS_48K: ZXSpecs = {
        ZXSpecsBuilder::new()
            .freq_cpu(3_500_000)
            .clocks_first_pixel(14336)
            .clocks_row(24, 128, 24, 48)
            .lines(48, 192, 48, 24)
            .contention([6, 5, 4, 3, 2, 1, 0, 0], 1)
            .interrupt_length(32)
            .build()
        };
}

/// Machine type
#[derive(Clone, Copy, Debug)]
pub enum ZXMachine {
    Sinclair48K,
    Sinclair128K,
}

impl ZXMachine {
    /// Returns current machine specs as ref to static value
    pub fn specs(self) -> &'static ZXSpecs {
        match self {
            ZXMachine::Sinclair48K => &SPECS_48K,
            // TODO: FIX
            ZXMachine::Sinclair128K => &SPECS_48K,
        }
    }

    /// Returns contention during specified time
    pub fn contention_clocks(self, clocks: u64) -> u64 {
        let specs = self.specs();
        if (clocks < (specs.clocks_first_pixel - 1)) ||
           (clocks >= (specs.clocks_first_pixel - 1) + specs.lines_screen * specs.clocks_line) {
            return 0;
        }
        let clocks_trough_line = (clocks - (specs.clocks_first_pixel - 1)) % specs.clocks_line;
        if clocks_trough_line >= specs.clocks_screen_row {
            return 0;
        }
        return self.specs().contention_pattern[(clocks_trough_line % 8) as usize];
    }

    /// Checks port contention on machine
    pub fn port_is_contended(self, port: u16) -> bool {
        match self {
            ZXMachine::Sinclair48K => {
                // every even port
                (port & 0x0001) == 0
            }
            ZXMachine::Sinclair128K => false,
        }
    }

    /// Checks address contention on machine
    pub fn addr_is_contended(self, addr: u16) -> bool {
        // how this works for other machines?
        (addr >= 0x4000) && (addr < 0x8000)
    }
}
