use utils::*;
/// Z80 processor System bus
/// Implement it for communication with CPU.
#[allow(unused_variables)]
pub trait Z80Bus {
    /// Required method for read byte from bus
    fn read(&self, addr: u16) -> u8;
    /// Required method for write byte to bus
    fn write(&mut self, addr: u16, data: u8);
    // Method for reading from io port. Default implementation is empty
    fn read_io(&self, addr: u16) -> u8 {
        0
    }
    // Method for writing to io port. Default implementation is empty
    fn write_io(&mut self, addr: u16, data: u8) {

    }
    /// provided metod to write word, LSB first
    fn write_word(&mut self, addr: u16, data: u16) {
        let (h, l) = split_word(data);
        self.write(addr, l);
        self.write(addr.wrapping_add(1), h);
    }
    /// provided method to read word
    fn read_word(&self, addr: u16) -> u16 {
        let l = self.read(addr);
        let h = self.read(addr.wrapping_add(1));
        make_word(h, l)
    }

    /// mut bacause on interrupt read some internal system attributes
    /// may be changed
    fn read_interrupt(&mut self) -> u8 {
        0
    }
    // /// method, invoked by  Z80 to check NMI signal
    // fn nmi(&mut self) -> bool {
    //     false
    // }
    //
    // /// method, invoked by Z80 to check INT signal
    // fn int(&mut self) -> bool {
    //     false
    // }

    /// method, invoked by Z80 in case of RETI instruction. Default implementation is empty
    fn reti(&mut self) {

    }

    /// method, invoked by Z80 in case of HALT line change
    fn halt(&mut self, halted: bool) {

    }
}
