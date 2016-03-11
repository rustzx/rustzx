use z80::Z80;
use zx::ZXBus;

/// Freq of Computer
const ZX_FREQ: u64 = 4 * 1024 * 1024; // Hz

/// ZX Spectrum computer struct
pub struct ZXComputer {
    pub cpu: Z80,
    pub bus: ZXBus,
}
impl ZXComputer {
    pub fn new() -> ZXComputer {
        ZXComputer {
            cpu: Z80::new(),
            bus: ZXBus::new(),
        }
    }
    /// emulate max 100 ticks, just testing
    pub fn emulate(&mut self) {
        self.cpu.emulate(&mut self.bus);
    }
    /// load default rom, just testing
    pub fn load_default_rom(&mut self) {
        self.bus.load_rom("/home/pacmancoder/code/z80/main.rom");
    }
}
