use z80::Z80Bus;
use zx::ZXMemory;
use super::screen::*;
use utils::split_word;
use super::ZXKey;

const ULA_ATTR_CYCLE: u64 = 8;
const ULA_ATTR_PER_CYCLE: u64 = 2;
const ULA_ATTR_IDLE_BEGIN: u64 = 4;
const ULA_BYTES_PER_CYCLE: u64 = 2;
const ULA_ROW_RENDER_CLOCKS: u64 = 128;

pub enum ZXModel {
    Sinclair48K,
    Sinclair128K,
}

impl ZXModel {
    fn frame_clocks(&self) -> u64 {
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

    pub fn new_frame(&mut self) {
        self.frame_clocks = 0;
    }

    fn floating_bus_value(&self) -> u8 {
        // top border draw
        if self.frame_clocks < self.model.frame_clocks() {
            return 0xFF;
        }
        let clocks = self.frame_clocks - self.model.frame_clocks();
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
    fn read_io(&self, addr: u16) -> u8 {
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
            // TODO: Floating bus
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

    fn tell_clocks(&mut self, clocks: u64) {
        self.frame_clocks += clocks;
    }

    fn read_interrupt(&mut self) -> u8 {
        0xFF
    }
}
