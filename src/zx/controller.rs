use z80::{Z80, Z80Bus};
use zx::ZXMemory;

/// ZX System controller
pub struct ZXController {
    cpu: Option<Z80>,
    memory: Option<ZXMemory>,
}

impl ZXController {
    pub fn new() -> ZXController {
        ZXController {
            cpu: None,
            memory: None,
        }
    }

    pub fn attach_cpu(&mut self, cpu: Z80) {
        self.cpu = Some(cpu);
    }

    pub fn atach_memory(&mut self, memory: ZXMemory) {
        self.memory = Some(memory);
    }
}

impl Z80Bus for ZXController {
    fn read(&self, addr: u16) -> u8 {
        if let Some(ref memory) = self.memory {
            memory.read(addr)
        } else {
            0
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        if let Some(ref mut memory) = self.memory {
            memory.write(addr, data);
        };
    }
}
