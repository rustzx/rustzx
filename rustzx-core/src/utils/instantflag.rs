use core::cell::Cell;

/// Instant flag - type, which resets self on immutable read
pub struct InstantFlag {
    f: Cell<bool>,
}

impl InstantFlag {
    /// constructs self from initial value
    pub fn new(value: bool) -> InstantFlag {
        InstantFlag {
            f: Cell::new(value),
        }
    }

    /// immutable read with reset
    pub fn pick(&self) -> bool {
        let value = self.f.get();
        self.f.set(false);
        value
    }

    /// read, but not reset
    pub fn get_direct(&self) -> bool {
        self.f.get()
    }

    /// set flag
    pub fn set(&mut self) {
        self.f.set(true);
    }
}
