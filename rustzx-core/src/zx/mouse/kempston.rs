const WHEEL_MASK: u8 = 0xF0;
const WHEEL_SHIFT: usize = 4;

// non_exhaustive allows to restrict struct instantiation only to `Default::defaault`
#[non_exhaustive]
pub(crate) struct KempstonMouse {
    pub buttons_port: u8,
    pub x_pos_port: u8,
    pub y_pos_port: u8,
}

impl Default for KempstonMouse {
    fn default() -> Self {
        Self {
            buttons_port: 0xFF,
            x_pos_port: 0xFF,
            y_pos_port: 0xFF,
        }
    }
}

#[cfg_attr(feature = "strum", derive(strum::EnumIter))]
#[derive(Clone, Copy, Debug)]
pub enum KempstonMouseButton {
    Left = 0x01,
    Right = 0x02,
    Middle = 0x04,
    Additional = 0x08,
}

#[cfg_attr(feature = "strum", derive(strum::EnumIter))]
#[derive(Clone, Copy, Debug)]
#[repr(i8)]
pub enum KempstonMouseWheelDirection {
    Up = 1,
    Down = -1,
}

impl KempstonMouse {
    pub fn send_button(&mut self, button: KempstonMouseButton, pressed: bool) {
        if pressed {
            self.buttons_port &= !(button as u8);
            return;
        }

        self.buttons_port |= button as u8
    }

    pub fn send_wheel(&mut self, dir: KempstonMouseWheelDirection) {
        let mut current = (self.buttons_port & WHEEL_MASK) >> WHEEL_SHIFT;
        current = ((current as i8) + (dir as i8)) as u8;
        self.buttons_port =
            (self.buttons_port & (!WHEEL_MASK)) | ((current << WHEEL_SHIFT) & WHEEL_MASK);
    }

    pub fn send_pos_diff(&mut self, x: i8, y: i8) {
        self.x_pos_port = ((self.x_pos_port as i16) + x as i16) as u8;
        self.y_pos_port = ((self.y_pos_port as i16) - y as i16) as u8;
    }
}
