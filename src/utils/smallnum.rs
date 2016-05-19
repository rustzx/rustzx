#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
/// 1 bit unsigned
pub enum U1 {
    N0, N1,
}
impl U1 {
    /// Constructs self from byte
    /// # Panics
    /// Panics if assirtion `shift < 8` failed
    pub fn from_byte(value: u8, shift: u8) -> U1 {
        assert!(shift < 8);
        match (value >> shift) & 1 {
            0 => U1::N0,
            _ => U1::N1,
        }
    }
    /// Transforms self back to byte
    pub fn as_byte(self) -> u8 {
        match self {
            U1::N0 => 0,
            U1::N1 => 1,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
/// 2 bit unsigned
pub enum U2 {
    N0, N1, N2, N3,
}
impl U2 {
    /// Constructs self from byte
    /// # Panics
    /// Panics if assirtion `shift < 7` failed
    pub fn from_byte(value: u8, shift: u8) -> U2 {
        assert!(shift < 7);
        match (value >> shift) & 3 {
            0 => U2::N0,
            1 => U2::N1,
            2 => U2::N2,
            _ => U2::N3,
        }
    }
    /// Transforms self back to byte
    pub fn as_byte(self) -> u8 {
        match self {
            U2::N0 => 0,
            U2::N1 => 1,
            U2::N2 => 2,
            U2::N3 => 3,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(rustfmt, rustfmt_skip)]
/// 3 bit unsigned
pub enum U3 {
    N0, N1, N2, N3, N4, N5, N6, N7,
}
impl U3 {
    /// Constructs self from byte
    /// # Panics
    /// Panics if assirtion `shift < 6` failed
    pub fn from_byte(value: u8, shift: u8) -> U3 {
        assert!(shift < 6);
        match (value >> shift) & 7 {
            0 => U3::N0,
            1 => U3::N1,
            2 => U3::N2,
            3 => U3::N3,
            4 => U3::N4,
            5 => U3::N5,
            6 => U3::N6,
            _ => U3::N7,
        }
    }
    /// Transforms self back to byte
    pub fn as_byte(self) -> u8 {
        match self {
            U3::N0 => 0,
            U3::N1 => 1,
            U3::N2 => 2,
            U3::N3 => 3,
            U3::N4 => 4,
            U3::N5 => 5,
            U3::N6 => 6,
            U3::N7 => 7,
        }
    }
}
