use z80::Z80Bus;
use zx::ZXMemory;
use super::{ZXScreen};
use utils::split_word;
use super::ZXKey;

/// ZX System controller
pub struct ZXController {
    memory: Option<ZXMemory>,
    screen: Option<ZXScreen>,
    int: bool,
    keyboard: [u8; 8],
    border_color: u8,
}

impl ZXController {
    pub fn new() -> ZXController {
        ZXController {
            memory: None,
            screen: None,
            int: false,
            keyboard: [0xFF; 8],
            border_color: 0x00,
        }
    }

    pub fn atach_memory(&mut self, memory: ZXMemory) {
        self.memory = Some(memory);
    }

    pub fn attach_screen(&mut self, screen: ZXScreen) {
        self.screen = Some(screen);
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
                tmp
            }
            _ => 0x00,
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

    // fn int(&mut self) -> bool {
    //     let tmp = self.int;
    //     self.int = false;
    //     tmp
    // }
}
