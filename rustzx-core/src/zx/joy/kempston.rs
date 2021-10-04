/// Kempston key type. Port bit encoded in enum values
#[derive(Clone, Copy)]
pub enum KempstonKey {
    Right = 0x01,
    Left = 0x02,
    Down = 0x04,
    Up = 0x08,
    Fire = 0x10,
    /// Undocumented extra button 1
    Ext1 = 0x20,
    /// Undocumented extra button 2
    Ext2 = 0x40,
    /// Undocumented extra button 3
    Ext3 = 0x80,
}

/// Kempston Joystick
#[derive(Default)]
pub(crate) struct KempstonJoy {
    state: u8,
}

impl KempstonJoy {
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
