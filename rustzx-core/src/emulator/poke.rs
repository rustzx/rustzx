#[derive(Clone, Copy)]
pub enum PokeAction {
    Mem { addr: u16, value: u8 },
}

impl PokeAction {
    pub const fn mem(addr: u16, value: u8) -> Self {
        Self::Mem { addr, value }
    }
}

pub trait Poke {
    fn actions(&self) -> &[PokeAction];
}

// Skips `scroll?` message print in ROM (48K)
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
