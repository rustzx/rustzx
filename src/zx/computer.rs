extern crate time;

use cpu::Z80;
use zx::ZXBus;

/// Freq of Computer
const ZX_FREQ: u64 = 4 * 1024 * 1024; // Hz

/// ZX Spectrum computer struct
pub struct ZXComputer {
    cpu: Z80,
    bus: ZXBus,
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
        let t1 = time::precise_time_ns();
        self.cpu.emulate(&mut self.bus);
        let t2 = time::precise_time_ns();
        println!("Emulation time: {} ns", t2 - t1);
    }
    /// load default rom, just testing
    pub fn load_default_rom(&mut self) {
        self.bus.load_rom("/home/pacmancoder/code/z80/main.rom");
    }
}
