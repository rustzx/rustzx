/// Represents memory where opcodes generated by [`CodeGenerator`] will be placed
pub trait CodegenMemorySpace {
    fn write_byte(&mut self, addr: u16, byte: u8);

    fn write_word(&mut self, addr: u16, word: u16) {
        let [l, h] = word.to_le_bytes();
        self.write_byte(addr, l);
        self.write_byte(addr, h);
    }
}

/// Provides easy methods to generate some Z80 opcodes. Currently only limited
/// opcode list is supported
pub struct CodeGenerator<'a, Mem: CodegenMemorySpace> {
    mem: &'a mut Mem,
    current_addr: u16,
}

impl<'a, Mem: CodegenMemorySpace> CodeGenerator<'a, Mem> {
    pub fn new(mem: &'a mut Mem) -> Self {
        Self {
            mem,
            current_addr: 0x0000,
        }
    }

    /// Sets current base address to write opcodes
    pub fn codegen_set_addr(&mut self, addr: u16) -> &mut Self {
        self.current_addr = addr;
        self
    }

    /// Generates most optimal jump opcode generation from the current address
    pub fn jump(&mut self, addr: u16) -> &mut Self {
        // Currently only direct jump method is implemented
        self.exact_opcode_jump_direct(addr)
    }

    /// Generates direct jump opcode (0xC3)
    pub fn exact_opcode_jump_direct(&mut self, addr: u16) -> &mut Self {
        self.write_byte(0xC3);
        self.write_word(addr);
        self
    }

    fn write_byte(&mut self, byte: u8) {
        self.mem.write_byte(self.current_addr, byte);
        self.current_addr += 1;
    }

    fn write_word(&mut self, word: u16) {
        let [l, h] = word.to_le_bytes();
        self.write_byte(l);
        self.write_byte(h);
    }
}
