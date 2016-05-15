use utils::*;
use z80::Clocks;
/// Z80 processor System bus
/// Implement it for communication with CPU.
#[allow(unused_variables)]
pub trait Z80Bus {
    /// Required method for reading byte without waiting
    fn read_internal(&self, addr: u16) -> u8;
    /// Required method for write byte to bus without waiting
    fn write_internal(&mut self, addr: u16, data: u8);

    /// wait some clocks
    fn wait_mreq(&mut self, addr: u16, clk: Clocks);
    /// wait while mreq is not active (can be different from
    /// active mreq contention as it works in ZX Spectrum 2+/3)
    fn wait_no_mreq(&mut self, addr: u16, clk: Clocks);

    fn wait_internal(&mut self, clk: Clocks);
    /// any single clock (t-state) can cause contention on ULA
    /// or any other chipm which not detects MREQ signal
    fn wait_loop(&mut self, addr: u16, clk: Clocks) {
        for _ in 0..clk.count() {
            self.wait_no_mreq(addr, Clocks(1));
        }
    }

    // normal read from memory, contention may be applied
    fn read(&mut self, addr: u16, clk: Clocks) -> u8 {
        self.wait_mreq(addr, clk);
        self.read_internal(addr)
    }

    // normal write to memory, contention may be applied
    fn write(&mut self, addr: u16, value: u8, clk: Clocks) {
        self.wait_mreq(addr, clk);
        self.write_internal(addr, value)
    }

    // Method for reading from io port.
    fn read_io(&mut self, port: u16) -> u8;
    // Method for writing to io port.
    fn write_io(&mut self, port: u16, data: u8);

    /// provided metod to write word, LSB first (clk - clocks per byte)
    fn write_word(&mut self, addr: u16, data: u16, clk: Clocks) {
        let (h, l) = split_word(data);
        self.write(addr, l, clk);
        self.write(addr.wrapping_add(1), h, clk);
    }
    /// provided method to read word (clk - clocks per byte)
    fn read_word(&mut self, addr: u16, clk: Clocks) -> u16 {
        let l = self.read(addr, clk);
        let h = self.read(addr.wrapping_add(1), clk);
        make_word(h, l)
    }

    /// mut bacause on interrupt read some internal system attributes
    /// may be changed
    fn read_interrupt(&mut self) -> u8;

    /// method, invoked by Z80 in case of RETI instruction. Default implementation is empty
    fn reti(&mut self);

    /// method, invoked by Z80 in case of HALT line change
    fn halt(&mut self, halted: bool);

    fn int_active(&self) -> bool;
    fn nmi_active(&self) -> bool;
}
