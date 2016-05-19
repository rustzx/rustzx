//! Contains ZX Spectrum System contrller (like lua or so) of emulator
//! TODO: Make ZXController Builder

use z80::{Z80Bus, Clocks};
use zx::ZXMemory;
use zx::machine::ZXMachine;
use super::screen::*;
use super::ZXKey;
use utils::split_word;

/// ZX System controller
pub struct ZXController {
    machine: ZXMachine,
    memory: Option<ZXMemory>,
    screen: Option<ZXScreen>,
    keyboard: [u8; 8],
    border_color: u8,
    ear: bool,
    frame_clocks: u64,
}

impl ZXController {
    /// Returns new ZXController
    pub fn new(machine: ZXMachine) -> ZXController {
        ZXController {
            machine: machine,
            memory: None,
            screen: None,
            keyboard: [0xFF; 8],
            border_color: 0x00,
            ear: true,
            frame_clocks: 0,
        }
    }
    /// Captures ZXMemory
    pub fn atach_memory(&mut self, memory: ZXMemory) {
        self.memory = Some(memory);
    }

    /// Captures ZXScreen
    pub fn attach_screen(&mut self, screen: ZXScreen) {
        self.screen = Some(screen);
    }

    /// Changes ear bit
    pub fn set_ear(&mut self, value: bool) {
        self.ear = value;
    }

    /// Returns Screen texture
    /// # Panics
    /// Panics when screen is not assigned
    pub fn get_screen_texture(&self) -> &[u8] {
        if let Some(ref screen) = self.screen {
            screen.clone_texture()
        } else {
            panic!("screen is not assigned to controller");
        }
    }

    /// get current border color
    pub fn get_border_color(&self) -> u8 {
        self.border_color
    }

    /// get clocks, passed from frame
    /// TODO: Use `Clocks` struct ?
    pub fn get_frame_clocks(&self) -> u64 {
        self.frame_clocks
    }

    /// Changes key state in controller
    pub fn send_key(&mut self, key: ZXKey, pressed: bool) {
        // TODO: Move row detection to ZXKey type
        let rownum = match key.half_port {
            0xFE => Some(0),
            0xFD => Some(1),
            0xFB => Some(2),
            0xF7 => Some(3),
            0xEF => Some(4),
            0xDF => Some(5),
            0xBF => Some(6),
            0x7F => Some(7),
            _ => None,
        };
        if let Some(rownum) = rownum {
            self.keyboard[rownum] = self.keyboard[rownum] & (!key.mask);
            if !pressed {
                self.keyboard[rownum] |= key.mask;
            }
        }
    }

    /// Dumps memory space
    pub fn dump(&self) -> Vec<u8> {
        if let Some(ref mem) = self.memory {
            mem.dump()
        } else {
            Vec::new()
        }
    }

    /// Returns current bus floating value
    fn floating_bus_value(&self) -> u8 {
        let specs = self.machine.specs();
        let clocks = self.frame_clocks;
        if clocks < 14338 {
            return 0xFF;
        }
        let clocks = self.frame_clocks - 14338;
        let row = clocks / specs.clocks_line;
        let clocks = clocks % specs.clocks_line;
        let col = (clocks / 8) * 2 + (clocks % 8) / 2;
        if row < 192 && clocks < 124 && ((clocks & 0x04) == 0) {
            if let Some(ref mem) = self.memory {
                if clocks % 2 == 0 {
                    return mem.read(get_bitmap_line_addr(row as u16) + col as u16);
                } else {
                    let byte = (row / 8) * 32 + col;
                    return mem.read(0x5800 + byte as u16);
                };
            }
        }
        return 0xFF;
    }

    /// Returns early IO contention clocks
    fn io_contention_first(&mut self, port: u16) {
        if self.machine.addr_is_contended(port) {
            self.frame_clocks += self.machine.contention_clocks(self.frame_clocks);
        };
        self.frame_clocks += 1;
    }

    /// Returns late IO contention clocks
    fn io_contention_last(&mut self, port: u16) {
        if self.machine.port_is_contended(port) {
            self.frame_clocks += self.machine.contention_clocks(self.frame_clocks);
            self.frame_clocks += 2;
        } else {
            if self.machine.addr_is_contended(port) {
                self.frame_clocks += self.machine.contention_clocks(self.frame_clocks);
                self.frame_clocks += 1;
                self.frame_clocks += self.machine.contention_clocks(self.frame_clocks);
                self.frame_clocks += 1;
                self.frame_clocks += self.machine.contention_clocks(self.frame_clocks);
            } else {
                self.frame_clocks += 2;
            }
        }
    }

    /// Starts a new frame
    pub fn new_frame(&mut self) {
        if self.frame_clocks >= self.machine.specs().clocks_frame {
            self.frame_clocks -= self.machine.specs().clocks_frame
        }
        if let Some(ref mut scr) = self.screen {
            scr.new_frame();
        }
    }

    /// Returns true if all frame clocks has been passed
    pub fn frame_finished(&self) -> bool {
        self.frame_clocks >= self.machine.specs().clocks_frame
    }

    /// Returns current clocks from frame start
    pub fn clocks(&self) -> u64 {
        self.frame_clocks
    }
}

impl Z80Bus for ZXController {
    fn read_internal(&self, addr: u16) -> u8 {
        if let Some(ref memory) = self.memory {
            memory.read(addr)
        } else {
            0
        }
    }

    fn write_internal(&mut self, addr: u16, data: u8) {
        if let Some(ref mut memory) = self.memory {
            memory.write(addr, data);
            match addr {
                0x4000...0x57FF => {
                    if let Some(ref mut screen) = self.screen {
                        screen.write_bitmap_byte(addr, Clocks(self.frame_clocks as usize), data);
                    }
                }
                0x5800...0x5AFF => {
                    if let Some(ref mut screen) = self.screen {
                        screen.write_attr_byte(addr, Clocks(self.frame_clocks as usize), data);
                    }
                }
                _ => {}
            }
        };
    }

    fn wait_internal(&mut self, clk: Clocks) {
        self.frame_clocks += clk.count() as u64;
    }

    fn wait_mreq(&mut self, addr: u16, clk: Clocks) {
        match self.machine {
            ZXMachine::Sinclair48K => {
                // contention in low 16k RAM
                if self.machine.addr_is_contended(addr) {
                    let last_clocks = self.frame_clocks;
                    self.frame_clocks += self.machine.contention_clocks(last_clocks);
                }
            }
            _ => {}
        }
        self.frame_clocks += clk.count() as u64;
    }

    fn wait_no_mreq(&mut self, addr: u16, clk: Clocks) {
        match self.machine {
            ZXMachine::Sinclair48K => {
                // contention in low 16k RAM
                if self.machine.addr_is_contended(addr) {
                    let last_clocks = self.frame_clocks;
                    self.frame_clocks += self.machine.contention_clocks(last_clocks);
                }
            }
            _ => {}
        }
        self.frame_clocks += clk.count() as u64;
    }

    fn read_io(&mut self, port: u16) -> u8 {
        // all contentions check
        self.io_contention_first(port);
        self.io_contention_last(port);
        // find out what we need to do
        let (h, _) = split_word(port);
        let output = if port & 0x0001 == 0 {
            let mut tmp: u8 = 0xFF;
            for n in 0..8 {
                // if bit of row reset
                if ((h >> n) & 0x01) == 0 {
                    tmp &= self.keyboard[n];
                }
            }
            // invert bit 6 if ear active;
            if self.ear {
                tmp ^= 0x40;
            }
            // 5 and 7 unused
            tmp
        } else {
            self.floating_bus_value()
        };
        // add one clock after operation
        self.frame_clocks += 1;
        output
    }

    fn write_io(&mut self, port: u16, data: u8) {
        // first contention
        self.io_contention_first(port);
        // if port from lua
        if port & 0x0001 == 0 {
            self.border_color = data & 0x07;
        }
        // last contention after byte write
        self.io_contention_last(port);
        // add one clock after operation
        self.frame_clocks += 1;
    }

    fn read_interrupt(&mut self) -> u8 {
        0xFF
    }

    fn int_active(&self) -> bool {
        self.frame_clocks % self.machine.specs().clocks_frame <
        self.machine.specs().interrupt_length
    }

    fn nmi_active(&self) -> bool {
        false
    }
    fn reti(&mut self) {}

    fn halt(&mut self, _: bool) {}
}
