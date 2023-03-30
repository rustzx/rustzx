//! Pokes are used to modify internal emulator state such as memory, registers, etc.

/// Action to perform on emulator state
#[derive(Clone, Copy)]
pub enum PokeAction {
    Mem { addr: u16, value: u8 },
}

impl PokeAction {
    /// Creates new memory poke action
    pub const fn mem(addr: u16, value: u8) -> Self {
        Self::Mem { addr, value }
    }
}

pub trait Poke {
    /// Returns list of actions to perform on emulator state
    fn actions(&self) -> &[PokeAction];
}

/// Poke which disables message and enter key prompt in 48K ROM when scrolling screen in BASIC mode
pub struct DisableScrollMessageRom48;
impl Poke for DisableScrollMessageRom48 {
    fn actions(&self) -> &[PokeAction] {
        // Injects `JP 0x0CD2` at 0x0C88
        // https://skoolkid.github.io/rom/asm/0C55.html
        const ACTIONS: &[PokeAction] = &[
            PokeAction::mem(0x0C88, 0xC3),
            PokeAction::mem(0x0C89, 0xD2),
            PokeAction::mem(0x0C8A, 0x0C),
        ];

        ACTIONS
    }
}
