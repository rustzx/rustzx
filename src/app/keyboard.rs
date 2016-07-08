//! Provides function `vkey_to_zxkey` for translatng glutin key to internal format enum
// Key type passed to main function
use glium::glutin::VirtualKeyCode as VKey;
// emulator key-related stuff
use zx::keys::*;

/// returns `ZXKey` from glutin `VirtualKeyCode`
pub fn vkey_to_zxkey(key: VKey) -> Option<ZXKey> {
    match key {
        // FEFE
        VKey::LShift | VKey::RShift => Some(ZX_KEY_SHIFT),
        VKey::Z => Some(ZX_KEY_Z),
        VKey::X => Some(ZX_KEY_X),
        VKey::C => Some(ZX_KEY_C),
        VKey::V => Some(ZX_KEY_V),
        // FDDE
        VKey::A => Some(ZX_KEY_A),
        VKey::S => Some(ZX_KEY_S),
        VKey::D => Some(ZX_KEY_D),
        VKey::F => Some(ZX_KEY_F),
        VKey::G => Some(ZX_KEY_G),
        // FBFE
        VKey::Q => Some(ZX_KEY_Q),
        VKey::W => Some(ZX_KEY_W),
        VKey::E => Some(ZX_KEY_E),
        VKey::R => Some(ZX_KEY_R),
        VKey::T => Some(ZX_KEY_T),
        // F7FE
        VKey::Key1 => Some(ZX_KEY_1),
        VKey::Key2 => Some(ZX_KEY_2),
        VKey::Key3 => Some(ZX_KEY_3),
        VKey::Key4 => Some(ZX_KEY_4),
        VKey::Key5 => Some(ZX_KEY_5),
        // EFFE
        VKey::Key0 => Some(ZX_KEY_0),
        VKey::Key9 => Some(ZX_KEY_9),
        VKey::Key8 => Some(ZX_KEY_8),
        VKey::Key7 => Some(ZX_KEY_7),
        VKey::Key6 => Some(ZX_KEY_6),
        // DFFE
        VKey::P => Some(ZX_KEY_P),
        VKey::O => Some(ZX_KEY_O),
        VKey::I => Some(ZX_KEY_I),
        VKey::U => Some(ZX_KEY_U),
        VKey::Y => Some(ZX_KEY_Y),
        // BFFE
        VKey::Return => Some(ZX_KEY_ENTER),
        VKey::L => Some(ZX_KEY_L),
        VKey::K => Some(ZX_KEY_K),
        VKey::J => Some(ZX_KEY_J),
        VKey::H => Some(ZX_KEY_H),
        // 7FFE
        VKey::Space => Some(ZX_KEY_SPACE),
        VKey::LControl | VKey::RControl => Some(ZX_KEY_SYM_SHIFT),
        VKey::M => Some(ZX_KEY_M),
        VKey::N => Some(ZX_KEY_N),
        VKey::B => Some(ZX_KEY_B),
        _ => None,
    }
}
