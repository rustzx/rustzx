use zx::machine::ZXMachine;
use zx::sound::ay::ZXAYMode;
use zx::constants::{SCREEN_WIDTH, SCREEN_HEIGHT};
// TODO: move comand line parsing here

/// Structure to handle all emulator runtime settings
pub struct RustzxSettings {
    pub screen_size: (usize, usize),
    pub machine: ZXMachine,
    pub ay_mode: ZXAYMode,
    pub ay_enabled: bool,
    pub beeper_enabled: bool,
    pub volume: usize,
    pub latency: usize,
    pub kempston: bool,
}

impl RustzxSettings {
    /// constructs new Settings
    pub fn new() -> RustzxSettings {
        RustzxSettings {
            screen_size: (SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2),
            machine: ZXMachine::Sinclair48K,
            ay_mode: ZXAYMode::Mono,
            ay_enabled: false,
            beeper_enabled: true,
            volume: 100,
            latency: 1024,
            kempston: false,
        }
    }
    /// Changes machine type
    pub fn machine(&mut self, machine: ZXMachine) -> &mut Self {
        self.machine = machine;
        match machine {
            ZXMachine::Sinclair48K => self.ay_enabled = false,
            ZXMachine::Sinclair128K => self.ay_enabled = true,
        }
        self
    }
    /// Changes screen size
    pub fn screen(&mut self, width: usize, height: usize) -> &mut Self {
        self.screen_size = (width, height);
        self
    }
    pub fn latency(&mut self, latency: usize) -> &mut Self {
        self.latency = latency;
        self
    }
    /// Changes AY chip mode
    pub fn ay_mode(&mut self, mode: ZXAYMode) -> &mut Self {
        self.ay_enabled = true;
        self.ay_mode = mode;
        self
    }
    /// Changes ay state (on/off)
    pub fn ay(&mut self, state: bool) -> &mut Self {
        self.ay_enabled = state;
        self
    }
    /// Changes beeper state (on/off)
    pub fn beeper(&mut self, state: bool) -> &mut Self {
        self.beeper_enabled = state;
        self
    }
    /// Changes volume
    pub fn volume(&mut self, val: usize) -> &mut Self {
        self.volume = if val > 200 {
            200
        } else {
            val
        };
        self
    }
    pub fn use_kempston(&mut self) -> &mut Self {
        self.kempston = true;
        self
    }
}
