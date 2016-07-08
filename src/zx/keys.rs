//! Module with hardware key port\masks
#![cfg_attr(rustfmt, rustfmt_skip)]

/// Struct, which contains mast and port of key
pub struct ZXKey {
    pub half_port: u8,
    pub mask: u8,
}
// 0xFEFE
pub const ZX_KEY_SHIFT: ZXKey = ZXKey {half_port: 0xFE, mask: 0x01};
pub const ZX_KEY_Z: ZXKey = ZXKey {half_port: 0xFE, mask: 0x02};
pub const ZX_KEY_X: ZXKey = ZXKey {half_port: 0xFE, mask: 0x04};
pub const ZX_KEY_C: ZXKey = ZXKey {half_port: 0xFE, mask: 0x08};
pub const ZX_KEY_V: ZXKey = ZXKey {half_port: 0xFE, mask: 0x10};
// FDFE
pub const ZX_KEY_A: ZXKey = ZXKey {half_port: 0xFD, mask: 0x01};
pub const ZX_KEY_S: ZXKey = ZXKey {half_port: 0xFD, mask: 0x02};
pub const ZX_KEY_D: ZXKey = ZXKey {half_port: 0xFD, mask: 0x04};
pub const ZX_KEY_F: ZXKey = ZXKey {half_port: 0xFD, mask: 0x08};
pub const ZX_KEY_G: ZXKey = ZXKey {half_port: 0xFD, mask: 0x10};
// FBFE
pub const ZX_KEY_Q: ZXKey = ZXKey {half_port: 0xFB, mask: 0x01};
pub const ZX_KEY_W: ZXKey = ZXKey {half_port: 0xFB, mask: 0x02};
pub const ZX_KEY_E: ZXKey = ZXKey {half_port: 0xFB, mask: 0x04};
pub const ZX_KEY_R: ZXKey = ZXKey {half_port: 0xFB, mask: 0x08};
pub const ZX_KEY_T: ZXKey = ZXKey {half_port: 0xFB, mask: 0x10};
// F7FE
pub const ZX_KEY_1: ZXKey = ZXKey {half_port: 0xF7, mask: 0x01};
pub const ZX_KEY_2: ZXKey = ZXKey {half_port: 0xF7, mask: 0x02};
pub const ZX_KEY_3: ZXKey = ZXKey {half_port: 0xF7, mask: 0x04};
pub const ZX_KEY_4: ZXKey = ZXKey {half_port: 0xF7, mask: 0x08};
pub const ZX_KEY_5: ZXKey = ZXKey {half_port: 0xF7, mask: 0x10};
// EFFE
pub const ZX_KEY_0: ZXKey = ZXKey {half_port: 0xEF, mask: 0x01};
pub const ZX_KEY_9: ZXKey = ZXKey {half_port: 0xEF, mask: 0x02};
pub const ZX_KEY_8: ZXKey = ZXKey {half_port: 0xEF, mask: 0x04};
pub const ZX_KEY_7: ZXKey = ZXKey {half_port: 0xEF, mask: 0x08};
pub const ZX_KEY_6: ZXKey = ZXKey {half_port: 0xEF, mask: 0x10};
//DFFE
pub const ZX_KEY_P: ZXKey = ZXKey {half_port: 0xDF, mask: 0x01};
pub const ZX_KEY_O: ZXKey = ZXKey {half_port: 0xDF, mask: 0x02};
pub const ZX_KEY_I: ZXKey = ZXKey {half_port: 0xDF, mask: 0x04};
pub const ZX_KEY_U: ZXKey = ZXKey {half_port: 0xDF, mask: 0x08};
pub const ZX_KEY_Y: ZXKey = ZXKey {half_port: 0xDF, mask: 0x10};
// BFFE
pub const ZX_KEY_ENTER: ZXKey = ZXKey {half_port: 0xBF, mask: 0x01};
pub const ZX_KEY_L: ZXKey = ZXKey {half_port: 0xBF, mask: 0x02};
pub const ZX_KEY_K: ZXKey = ZXKey {half_port: 0xBF, mask: 0x04};
pub const ZX_KEY_J: ZXKey = ZXKey {half_port: 0xBF, mask: 0x08};
pub const ZX_KEY_H: ZXKey = ZXKey {half_port: 0xBF, mask: 0x10};
// 7FFE
pub const ZX_KEY_SPACE: ZXKey = ZXKey {half_port: 0x7F, mask: 0x01};
pub const ZX_KEY_SYM_SHIFT: ZXKey = ZXKey {half_port: 0x7F, mask: 0x02};
pub const ZX_KEY_M: ZXKey = ZXKey {half_port: 0x7F, mask: 0x04};
pub const ZX_KEY_N: ZXKey = ZXKey {half_port: 0x7F, mask: 0x08};
pub const ZX_KEY_B: ZXKey = ZXKey {half_port: 0x7F, mask: 0x10};
