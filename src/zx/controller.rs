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
    // fn screen_clocks(&self) -> u64 {
    //     match *self {
    //         ZXModel::Sinclair48K => 191 * 224 + 128,
    //         ZXModel::Sinclair128K => 70908,
    //     }
    // }
    fn clocks_per_frame(&self) -> u64 {
        match *self {
            ZXModel::Sinclair48K => 69888,
            ZXModel::Sinclair128K => 70908,
        }
    }
    // TODO: RENAME
    fn first_pixel_clocks(&self) -> u64 {
        match *self {
            ZXModel::Sinclair48K => 14347,
            ZXModel::Sinclair128K => 14368,
        }
    }
    fn row_clocks(&self) -> u64 {
        match *self {
            ZXModel::Sinclair48K => 224,
            ZXModel::Sinclair128K => 228,
        }
    }

    // fn sceen_start_clocks(&self) -> u64 {
    //     match *self {
    //         ZXModel::Sinclair48K => 14335,
    //         // TODO: CHECK TIMINGS
    //         ZXModel::Sinclair128K => 14356,
    //     }
    // }
    fn contention_clocks(&self, clocks: u64) -> u64 {
        match *self {
            ZXModel::Sinclair48K => {
                if (clocks < 14335) || (clocks > 14335 + 191 * 224 + 128) {
                        return 0;
                }
                let clocks_trough_line = (clocks - 14335) % 224;
                if clocks_trough_line >= 128 {
                    return 0;
                }
                return ULA_48K_CONTENTION_PATTERN[(clocks_trough_line % 8) as usize];
            },
            ZXModel::Sinclair128K => {
                // make rustc happy
                return 0;
            },
        }
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
        // top border draw
        if self.frame_clocks < self.model.first_pixel_clocks() {
            return 0xFF;
        }
        let clocks = self.frame_clocks - self.model.first_pixel_clocks();
        // botttom border draw
        if clocks >= self.model.row_clocks() * SCREEN_HEIGHT as u64 {
            return 0xFF;
        }
        // TStates 4..7 is IDLE
        if clocks % ULA_ATTR_CYCLE >= ULA_ATTR_IDLE_BEGIN {
            // IDLE
            return 0xFF;
        }
        // if side border rendering
        if clocks % self.model.row_clocks() >= ULA_ROW_RENDER_CLOCKS  {
            // IDLE, ULA draws border
            return 0xFF;
        }
        // column is mod of clocks on clocks per row devided on CYCLE and multiplied by 2
        // and then plus zero or 1 col
        let col = ((clocks % self.model.row_clocks()) / ULA_ATTR_CYCLE) * ULA_BYTES_PER_CYCLE +
            (clocks % ULA_ATTR_CYCLE) / 2;
        // row is just clocks devided by clocks per row
        let row = clocks / self.model.row_clocks();
        // bitmap if clocks parity is even
        let is_bitmap  = clocks % 2 == 0;
        if is_bitmap {
            let byte = row * BYTES_PER_ROW + col;
            if let Some(ref mem) = self.memory {
                return mem.read(0x4000 + byte as u16);
            } else {
                return 0xFF;
            }
        } else {
            let byte = row / ROWS_PER_ATTR + col;
            if let Some(ref mem) = self.memory {
                return mem.read(0x5800 + byte as u16);
            } else {
                return 0xFF;
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
                if (addr >= 0x4000) && (addr <= 0x7fff) {
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
                if (addr >= 0x4000) && (addr <= 0x7fff) {
                    let last_clocks = self.frame_clocks;
                    self.frame_clocks += self.model.contention_clocks(last_clocks);
                }
            }
            _ => {}
        }
        self.frame_clocks += clk.count() as u64;
    }

    fn read_io(&mut self, addr: u16) -> u8 {
        let (h, l) = split_word(addr);
        match l {
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
                tmp
            }
            _ => {
                self.floating_bus_value()
            },
        }
    }

    fn write_io(&mut self, addr: u16, data: u8) {
        let (_, l) = split_word(addr);
        match l {
            0xFE => {
                self.border_color = data & 0x07;
            }
            _ => {}
        };
    }

    fn read_interrupt(&mut self) -> u8 {
        0xFF
    }
    fn int_active(&mut self) -> bool {
        self.frame_clocks < 32
    }
    fn nmi_active(&mut self) -> bool {
        false
    }
    fn reti(&mut self) {}
    fn halt(&mut self, _: bool) {}

}
