use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct EmulationEvents: u8 {
        const TAPE_FAST_LOAD_TRIGGER_DETECTED = 0b00000001;
    }
}

impl EmulationEvents {
    pub fn clear(&mut self) {
        self.bits = 0;
    }
}
