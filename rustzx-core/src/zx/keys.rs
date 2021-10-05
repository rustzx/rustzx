//! Module with hardware key port\masks
use strum::EnumIter;

/// Struct, which contains mast and port of key
#[rustfmt::skip]
#[derive(Clone, Copy, EnumIter)]
pub enum ZXKey {
    // Port 0xFEFE
    Shift, Z, X, C, V,
    // Port 0xFDFE
    A, S, D, F, G,
    // Port 0xFBFE
    Q, W, E, R, T,
    // Port 0xF7FE
    N1, N2, N3, N4, N5,
    // Port 0xEFFE
    N0, N9, N8, N7, N6,
    // Port 0xDFFE
    P, O, I, U, Y,
    // Port 0xBFFE
    Enter, L, K, J, H,
    // Port 0x7FFE
    Space, SymShift, M, N, B,
}

#[derive(Clone, Copy, EnumIter)]
pub enum CompoundKey {
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    CapsLock,
    Delete,
    Break,
}

impl CompoundKey {
    /// This mask is required to implement logic to keep
    /// modifier key pressed while one of the compound
    /// keys were released, but one of them is still
    /// pressed, therefore modifier should be unchanged
    pub(crate) fn modifier_mask(self) -> u32 {
        match self {
            CompoundKey::ArrowLeft => 0x00000001,
            CompoundKey::ArrowRight => 0x00000002,
            CompoundKey::ArrowUp => 0x00000004,
            CompoundKey::ArrowDown => 0x00000008,
            CompoundKey::CapsLock => 0x00000010,
            CompoundKey::Delete => 0x00000020,
            CompoundKey::Break => 0x00000040,
        }
    }

    pub(crate) fn modifier_key(self) -> ZXKey {
        ZXKey::Shift
    }

    pub(crate) fn primary_key(self) -> ZXKey {
        match self {
            CompoundKey::ArrowLeft => ZXKey::N5,
            CompoundKey::ArrowRight => ZXKey::N8,
            CompoundKey::ArrowUp => ZXKey::N7,
            CompoundKey::ArrowDown => ZXKey::N6,
            CompoundKey::CapsLock => ZXKey::N2,
            CompoundKey::Delete => ZXKey::N0,
            CompoundKey::Break => ZXKey::Space,
        }
    }
}

impl ZXKey {
    pub(crate) fn row_id(self) -> usize {
        match self.half_port() {
            0xFE => 0,
            0xFD => 1,
            0xFB => 2,
            0xF7 => 3,
            0xEF => 4,
            0xDF => 5,
            0xBF => 6,
            0x7F => 7,
            _ => unreachable!(),
        }
    }

    pub(crate) fn mask(&self) -> u8 {
        use ZXKey::*;
        match self {
            Shift | A | Q | N1 | N0 | P | Enter | Space => 0x01,
            Z | S | W | N2 | N9 | O | L | SymShift => 0x02,
            X | D | E | N3 | N8 | I | K | M => 0x04,
            C | F | R | N4 | N7 | U | J | N => 0x08,
            V | G | T | N5 | N6 | Y | H | B => 0x10,
        }
    }

    fn half_port(self) -> u8 {
        use ZXKey::*;
        match self {
            Shift | Z | X | C | V => 0xFE,
            A | S | D | F | G => 0xFD,
            Q | W | E | R | T => 0xFB,
            N1 | N2 | N3 | N4 | N5 => 0xF7,
            N0 | N9 | N8 | N7 | N6 => 0xEF,
            P | O | I | U | Y => 0xDF,
            Enter | L | K | J | H => 0xBF,
            Space | SymShift | M | N | B => 0x7F,
        }
    }
}
