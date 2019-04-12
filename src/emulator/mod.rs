//! Platform-independent high-level Emulator interaction module
use std::time::{Instant, Duration};
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
        let mut controller = ZXController::new(&settings);
        if let Some(ref path) = settings.rom {
            controller.load_rom(path);
        } else {
            controller.load_default_rom();
        };
        if let Some(ref path) = settings.tap {
            controller.tape.insert(path);
        }
        let mut out = Emulator {
            cpu: Z80::new(),
            controller: controller,
            speed: settings.speed,
            fast_load: settings.fastload,
            sound_enabled: settings.sound_enabled,
        };
        if let Some(ref path) = settings.sna {
            out.load_sna(path)
        }
        out
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
    pub fn emulate_frames(&mut self, max_time: Duration) -> Duration {
        let mut time = Duration::new(0, 0);
        'frame: loop {
            // start of current frame
            let start_time = Instant::now();
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
                        return start_time.elapsed();
                    };
                    // if speed is maximal.
                } else {
                    // if any frame passed then break cpu loop, but try to start new frame
                    if self.controller.frames_count() != 0 {
                        break 'cpu;
                    }
                }
            }
            time += start_time.elapsed();
            // if time is bigger than `max_time` then stop emulation cycle
            if time > max_time {
                break 'frame;
            }
        }
        self.controller.clear_events();
        return time;
    }
}
