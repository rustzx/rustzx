use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct EmulationEvents: u8 {
        const TAPE_FAST_LOAD_TRIGGER_DETECTED = 0b00000001;
        const PC_BREAKPOINT = 0b00000010;
    }
}

impl EmulationEvents {
    pub fn take(&mut self) -> Self {
        let events = *self;
        self.bits = 0;
        events
    }
}
