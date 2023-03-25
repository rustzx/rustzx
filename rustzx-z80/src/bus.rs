use crate::{
    opcode::{Opcode, Prefix},
    CodegenMemorySpace,
};

/// Z80 processor System bus
/// Implement it for communication with CPU.
pub trait Z80Bus {
    /// Required method for reading byte without waiting
    /// pass `self` as mut, because method can change state of
    /// bus or something
    fn read_internal(&mut self, addr: u16) -> u8;
    /// Required method for write byte to bus without waiting
    fn write_internal(&mut self, addr: u16, data: u8);
    /// Wait some clocks
    fn wait_mreq(&mut self, addr: u16, clk: usize);
    /// Wait while mreq is not active (can be different from
    /// active mreq contention as it works in ZX Spectrum 2+/3)
    fn wait_no_mreq(&mut self, addr: u16, clk: usize);
    /// Internal wait, avoiding contentions
    fn wait_internal(&mut self, clk: usize);
    /// Any single clock (t-state) can cause contention on ULA
    /// or any other chipm which not detects MREQ signal
    fn wait_loop(&mut self, addr: u16, clk: usize) {
        for _ in 0..clk {
            self.wait_no_mreq(addr, 1);
        }
    }
    // Normal read from memory, contention may be applied
    fn read(&mut self, addr: u16, clk: usize) -> u8 {
        self.wait_mreq(addr, clk);
        self.read_internal(addr)
    }
    // Normal write to memory, contention may be applied
    fn write(&mut self, addr: u16, value: u8, clk: usize) {
        self.wait_mreq(addr, clk);
        self.write_internal(addr, value)
    }
    // Method for reading from io port.
    fn read_io(&mut self, port: u16) -> u8;
    // Method for writing to io port.
    fn write_io(&mut self, port: u16, data: u8);
    /// Provided method to write word, LSB first (clk - clocks per byte)
    fn write_word(&mut self, addr: u16, data: u16, clk: usize) {
        let [l, h] = data.to_le_bytes();
        self.write(addr, l, clk);
        self.write(addr.wrapping_add(1), h, clk);
    }
    /// Provided method to read word (clk - clocks per byte)
    fn read_word(&mut self, addr: u16, clk: usize) -> u16 {
        let l = self.read(addr, clk);
        let h = self.read(addr.wrapping_add(1), clk);
        u16::from_le_bytes([l, h])
    }
    /// Reads value from bus during interrupt.
    /// mutable because on interrupt read some internal system attributes may be changed
    fn read_interrupt(&mut self) -> u8;
    /// Method, invoked by Z80 in case of RETI instruction. Default implementation is empty
    fn reti(&mut self);
    /// Method, invoked by Z80 in case of HALT line change
    fn halt(&mut self, halted: bool);
    /// Checks int signal
    fn int_active(&self) -> bool;
    /// Checks nmi signal
    fn nmi_active(&self) -> bool;
    /// invokes breakpoints check on bus device
    fn pc_callback(&mut self, addr: u16);
    fn process_unknown_opcode(&mut self, _prefix: Prefix, _opcode: Opcode) {}
}

impl<T> CodegenMemorySpace for T
where
    T: Z80Bus,
{
    fn write_byte(&mut self, addr: u16, byte: u8) {
        // write, but does not perform any processing (zero clock cycles)
        self.write(addr, byte, 0);
    }
}
