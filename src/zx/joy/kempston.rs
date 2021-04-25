/// Kempston key type. Port bit encoded in enum values
pub enum KempstonKey {
    Right = 0x01,
    Left = 0x02,
    Down = 0x04,
    Up = 0x08,
    Fire = 0x10,
}

/// Kempston Joystick
pub struct KempstonJoy {
    state: u8,
}

impl KempstonJoy {
    /// Constructs new Kemoston
    pub fn new() -> Self {
        KempstonJoy { state: 0x00 }
    }

    /// Simulates key press/release
    pub fn key(&mut self, key: KempstonKey, state: bool) {
        if state {
            self.state |= key as u8;
        } else {
            self.state &= !(key as u8);
        }
    }

    /// Reads joy value
    pub fn read(&self) -> u8 {
        self.state
    }
}
