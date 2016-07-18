//! Platform-independent high-level Emulator interaction module
use time;

use utils::*;
use z80::*;
use zx::ZXController;
use settings::RustzxSettings;

mod loaders;

/// Represents main Emulator structure
pub struct Emulator {
    cpu: Z80,
    // direct access to controller devices and control methods
    pub controller: ZXController,
    speed: EmulationSpeed,
    fast_load: bool,
    sound_enabled: bool,
}

impl Emulator {
    /// Constructs new emulator
    /// # Arguments
    /// `settings` - emulator settings
    pub fn new(settings: &RustzxSettings) -> Emulator {
        Emulator {
            cpu: Z80::new(),
            controller: ZXController::new(&settings),
            speed: EmulationSpeed::Definite(1),
            fast_load: false,
            sound_enabled: true,
        }
    }

    /// changes emulation speed
    pub fn set_speed(&mut self, new_speed: EmulationSpeed) {
        self.speed = new_speed;
    }

    /// changes fast loading flag
    pub fn set_fast_load(&mut self, value: bool) {
        self.fast_load = value;
    }

    /// changes sound playback flag
    pub fn set_sound(&mut self, value: bool) {
        self.sound_enabled = value;
    }

    /// function for sound generation request check
    pub fn have_sound(&self) -> bool {
        // enable sound only if speed is normal
        if let EmulationSpeed::Definite(1) = self.speed {
            if self.sound_enabled {
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// loads snapshot file
    pub fn load_sna(&mut self, file: &str) {
        loaders::load_sna(self, file)
    }

    /// events processing function
    fn process_event(&mut self, event: Event) {
        let Event { kind: e, time: _ } = event;
        match e {
            // Fast tape loading found, use it
            EventKind::FastTapeLoad if self.controller.tape.can_fast_load() && self.fast_load => {
                loaders::fast_load_tap(self);
            }
            _ => {}
        }
    }

    // processes all events, happened at frame emulation cycle
    fn process_all_events(&mut self) {
        loop {
            if let Some(event) = self.controller.pop_event() {
                self.process_event(event);
            } else {
                break;
            }
        }
    }

    /// Emulate frames, maximum in `max_time` time, returns emulation time in nanoseconds
    /// in most cases time is max 1/50 of second, even when using
    /// loader acceleration
    pub fn emulate_frames(&mut self, max_time: u64) -> u64 {
        let mut time = 0u64;
        'frame: loop {
            // start of current frame
            let start_time = time::precise_time_ns();
            // reset controller internal frame counter
            self.controller.reset_frame_counter();
            'cpu: loop {
                // Emulation step. if instant event happened then accept in and execute
                if !self.cpu.emulate(&mut self.controller) {
                    if let Some(event) = self.controller.pop_event() {
                        self.process_event(event);
                    }
                }
                // If speed is defined
                if let EmulationSpeed::Definite(multiplier) = self.speed {
                    if self.controller.frames_count() >= multiplier {
                        // no more frames
                        self.controller.clear_events();
                        return time::precise_time_ns() - start_time;
                    };
                    // if speed is maximal.
                } else {
                    // if any frame passed then break cpu loop, but try to start new frame
                    if self.controller.frames_count() != 0 {
                        break 'cpu;
                    }
                }
            }
            time += time::precise_time_ns() - start_time;
            // if time is bigger than `max_time` then stop emulation cycle
            if time > max_time {
                break 'frame;
            }
        }
        self.controller.clear_events();
        return time;
    }
}
