use z80::{Z80Bus, Clocks};
use zx::ZXMemory;
use super::screen::*;
use utils::split_word;
use super::ZXKey;

const ULA_ATTR_CYCLE: u64 = 8;
const ULA_ATTR_PER_CYCLE: u64 = 2;
const ULA_ATTR_IDLE_BEGIN: u64 = 4;
const ULA_BYTES_PER_CYCLE: u64 = 2;
const ULA_ROW_RENDER_CLOCKS: u64 = 128;

const ULA_48K_CONTENTION_PATTERN: [u64; 8] = [6, 5, 4, 3, 2, 1, 0, 0];

pub enum ZXModel {
    Sinclair48K,
    Sinclair128K,
}

impl ZXModel {
    fn clocks_per_frame(&self) -> u64 {
        match *self {
            ZXModel::Sinclair48K => 69888,
            ZXModel::Sinclair128K => 70908,
        }
    }
    // TODO: RENAME
    fn first_pixel_clocks(&self) -> u64 {
        match *self {
            ZXModel::Sinclair48K => 14347 - 9,
            ZXModel::Sinclair128K => 14368 - 9,
        }
    }
    fn row_clocks(&self) -> u64 {
        match *self {
            ZXModel::Sinclair48K => 224,
            ZXModel::Sinclair128K => 228,
        }
    }

    fn contention_clocks(&self, clocks: u64) -> u64 {
        match *self {
            ZXModel::Sinclair48K => {
                if (clocks < 14335) || (clocks >= 14335 + 192 * 224) {
                        return 0;
                }
                let clocks_trough_line = (clocks - 14335) % 224;
                if clocks_trough_line >= 128 {
                    return 0;
                }
                return ULA_48K_CONTENTION_PATTERN[(clocks_trough_line % 8) as usize];
            },
            ZXModel::Sinclair128K => {
                // ...
                return 0;
            },
        }
    }

    fn port_is_contended(&self, port: u16) -> bool {
        match *self {
            ZXModel::Sinclair48K => {
                // every even post
                (port & 0x0001) == 0
            }
            ZXModel::Sinclair128K => false,
        }
    }

    fn addr_is_contended(&self, addr: u16) -> bool {
        // how this works for other machines?
        (addr >= 0x4000) && (addr <= 0x7FFF)
    }
}

/// ZX System controller
pub struct ZXController {
    model: ZXModel,
    memory: Option<ZXMemory>,
    screen: Option<ZXScreen>,
    int: bool,
    keyboard: [u8; 8],
    border_color: u8,
    ear: bool,
    frame_clocks: u64,
}

impl ZXController {
    pub fn new(computer_model: ZXModel) -> ZXController {
        ZXController {
            model: computer_model,
            memory: None,
            screen: None,
            int: false,
            keyboard: [0xFF; 8],
            border_color: 0x00,
            ear: true,
            frame_clocks: 0,
        }
    }

    pub fn atach_memory(&mut self, memory: ZXMemory) {
        self.memory = Some(memory);
    }

    pub fn attach_screen(&mut self, screen: ZXScreen) {
        self.screen = Some(screen);
    }

    pub fn set_ear(&mut self, value: bool) {
        self.ear = value;
    }

    pub fn get_screen_texture(&self) -> &[u8] {
        if let Some(ref screen) = self.screen {
            screen.clone_texture()
        } else {
            panic!("screen is not assigned to controller");
        }
    }

    pub fn get_border_color(&self) -> u8 {
        self.border_color
    }
    pub fn set_int(&mut self) {
        self.int = true;
    }
    pub fn reset_int(&mut self) {
        self.int = false;
    }

    pub fn get_frame_clocks(&self) -> u64 {
        self.frame_clocks
    }

    pub fn send_key(&mut self, key: ZXKey, pressed: bool) {
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

    pub fn dump(&self) -> Vec<u8> {
        if let Some(ref mem) = self.memory {
            mem.dump()
        } else {
            Vec::new()
        }
    }

    fn floating_bus_value(&self) -> u8 {
        // TODO: Find out how to calculate floating bus, this function is incorrect!
        if self.frame_clocks < self.model.first_pixel_clocks() {
            return 0xFF;
        }
        // clocks relative to screen start
        let clocks = self.frame_clocks - self.model.first_pixel_clocks();
        // row is just clocks devided by clocks per row
        let row = clocks / self.model.row_clocks();
        // botttom border draw
        if row >= 192 as u64 {
            return 0xFF;
        }

        // if side border rendering
        if clocks % self.model.row_clocks() >= 124  {
            // IDLE, ULA draws border
            return 0xFF;
        }

        // TStates 4..7 is IDLE
        if clocks % 8 >= 4 {
            // IDLE
            return 0xFF;
        }

        // column is mod of clocks on clocks per row devided on CYCLE and multiplied by 2
        // and then plus zero or 1 col
        let col = ((clocks % self.model.row_clocks()) / 8) * 2 + (clocks % 8) / 2;
        // bitmap if clocks parity is even
        if clocks % 2 == 0 {
            if let Some(ref mem) = self.memory {
                return mem.read( get_line_base(row as u16) + col as u16);
            } else {
                return 0xFF;
            }
        } else {
            let byte = (row / 8) * 32  + col;
            if let Some(ref mem) = self.memory {
                return mem.read(0x5800 + byte as u16);
            } else {
                return 0xFF;
            }
        }
    }

    fn io_contention_first(&mut self, port: u16) {
        if self.model.addr_is_contended(port) {
            self.frame_clocks += self.model.contention_clocks(self.frame_clocks);
        };
        self.frame_clocks += 1;
    }

    fn io_contention_last(&mut self, port: u16) {
        if self.model.port_is_contended(port) {
            self.frame_clocks += self.model.contention_clocks(self.frame_clocks);
            self.frame_clocks += 2;
        } else {
            if self.model.addr_is_contended(port) {
                self.frame_clocks += self.model.contention_clocks(self.frame_clocks);
                self.frame_clocks += 1;
                self.frame_clocks += self.model.contention_clocks(self.frame_clocks);
                self.frame_clocks += 1;
                self.frame_clocks += self.model.contention_clocks(self.frame_clocks);
            } else {
                self.frame_clocks += 2;
            }
        }
    }

    pub fn new_frame(&mut self) {
        if self.frame_clocks >= self.model.clocks_per_frame() {
            self.frame_clocks -= self.model.clocks_per_frame()
        }
    }

    pub fn frame_finished(&self) -> bool {
        self.frame_clocks >= self.model.clocks_per_frame()
    }

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
                        screen.write_bitmap_byte(addr, data);
                    }
                }
                0x5800...0x5AFF => {
                    if let Some(ref mut screen) = self.screen {
                        screen.write_attr_byte(addr, data);
                    }
                },
                _ => {},
            }
        };
    }

    fn wait_internal(&mut self, clk: Clocks) {
        self.frame_clocks += clk.count() as u64;
    }

    fn wait_mreq(&mut self, addr: u16, clk: Clocks) {
        match self.model {
            ZXModel::Sinclair48K => {
                // contention in low 16k RAM
                if self.model.addr_is_contended(addr) {
                    let last_clocks = self.frame_clocks;
                    self.frame_clocks += self.model.contention_clocks(last_clocks);
                }
            }
            _ => {}
        }
        self.frame_clocks += clk.count() as u64;
    }

    fn wait_no_mreq(&mut self, addr: u16, clk: Clocks) {
        match self.model {
            ZXModel::Sinclair48K => {
                // contention in low 16k RAM
                if self.model.addr_is_contended(addr) {
                    let last_clocks = self.frame_clocks;
                    self.frame_clocks += self.model.contention_clocks(last_clocks);
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
        let (h, l) = split_word(port);
        let output = match l {
            // keyboard
            0xFE => {
                let mut tmp: u8 = 0xFF;
                for n in 0..8 {
                    // if bit of row reset
                    if ((h >> n) & 0x01) == 0 {
                        tmp &= self.keyboard[n];
                    }
                };
                // ear input;
                if !self.ear {
                    tmp &= 0b10111111;
                };
                // 5 and 7 unused
                tmp
            }
            _ => {
                self.floating_bus_value()
            },
        };
        // add one clock after operation
        self.frame_clocks += 1;
        output
    }

    fn write_io(&mut self, port: u16, data: u8) {
        // first contention
        self.io_contention_first(port);
        let (_, l) = split_word(port);
        match l {
            0xFE => {
                self.border_color = data & 0x07;
            }
            _ => {}
        };
        // last contention after byte write
        self.io_contention_last(port);
        // add one clock after operation
        self.frame_clocks += 1;
    }

    fn read_interrupt(&mut self) -> u8 {
        0xFF
    }
    fn int_active(&mut self) -> bool {
        self.frame_clocks % self.model.clocks_per_frame() < 32
    }
    fn nmi_active(&mut self) -> bool {
        false
    }
    fn reti(&mut self) {}
    fn halt(&mut self, _: bool) {}

}
