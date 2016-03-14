use std::fs::File;
use std::io::Read;
use z80::Z80Bus;

/// 16K ROM SIZE
const ROM_SIZE: usize = 1024 * 16;
/// 48 RAM SIZE
const RAM_SIZE: usize = 1024 * 48;

/// ZX Spectrum System Bus
pub struct ZXBus {
    rom: Vec<u8>,
    ram: [u8; RAM_SIZE],
    halted: bool,
}

impl ZXBus {
    /// new ZXBus
    pub fn new() -> ZXBus {
        ZXBus {
            rom: Vec::new(),
            ram: [0; RAM_SIZE],
            halted: false,
        }
    }

    /// loads rom from file
    pub fn load_rom(&mut self, file: &str) {
        let mut file = File::open(file).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        self.rom = buffer;
    }

    pub fn cpu_halted(&self) -> bool {
        self.halted
    }
}

impl Z80Bus for ZXBus {
    fn write(&mut self, addr: u16, data: u8) {
        if addr as usize >= ROM_SIZE {
            self.ram[addr as usize - ROM_SIZE] = data;
        }
    }
    
    fn read(&self, addr: u16) -> u8 {
        if (addr as usize) < ROM_SIZE {
            if addr as usize >= self.rom.len() {
                0_u8
            } else {
                self.rom[addr as usize]
            }
        } else {
            self.ram[addr as usize - ROM_SIZE]
        }
    }

    #[allow(unused_variables)]
    fn write_io(&mut self, addr: u16, data: u8) {

    }

    #[allow(unused_variables)]
    fn read_io(&self, addr: u16) -> u8 {
        0xCC
    }

    fn halt(&mut self, halted: bool) {
        self.halted = halted;
    }
}
