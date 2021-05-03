//! Platform-independent high-level Emulator interaction module
mod loaders;

use crate::{
    host::{Host, LoadableAsset, Snapshot, Tape},
    utils::*,
    z80::*,
    zx::{
        joy::kempston::KempstonKey,
        sound::sample::SoundSample,
        tape::{Tap, TapeImpl},
        ZXController,
        ZXKey,
        ZXMachine,
    },
    Result,
};

use core::time::Duration;

/// Represents main Emulator structure
pub struct Emulator<H: Host> {
    pub host: H,
    cpu: Z80,
    // direct access to controller devices and control methods
    pub controller: ZXController<H>,
    speed: EmulationSpeed,
    fast_load: bool,
    sound_enabled: bool,
}

pub trait Stopwatch {
    fn reset(&mut self);
    fn measure(&self) -> Duration;
}

impl<H: Host> Emulator<H> {
    /// Constructs new emulator
    /// # Arguments
    /// `settings` - emulator settings
    pub fn from_host(host: H) -> Result<Self> {
        let settings = host.settings();

        let speed = settings.emulation_speed;
        let fast_load = settings.tape_fastload;
        let sound_enabled = settings.sound_enabled;

        let cpu = Z80::new();
        let controller = ZXController::<H>::new(settings);

        let mut this = Self {
            host,
            cpu,
            controller,
            speed,
            fast_load,
            sound_enabled,
        };

        // Load initial assets
        this.reload_rom()?;
        this.reload_snapshot()?;
        this.reload_tape()?;

        Ok(this)
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

    pub fn reload_snapshot(&mut self) -> Result<()> {
        match self.host.snapshot() {
            Some(Snapshot::Sna(asset)) => loaders::sna::load_sna(self, asset),
            None => Ok(()),
        }
    }

    pub fn reload_tape(&mut self) -> Result<()> {
        match self.host.tape() {
            Some(Tape::Tap(asset)) => {
                self.controller.tape = Tap::from_asset(asset)?.into();
            }
            None => {}
        }

        Ok(())
    }

    pub fn reload_rom(&mut self) -> Result<()> {
        match self.host.settings().machine {
            ZXMachine::Sinclair48K => {
                if let Some(mut asset) = self.host.rom(0) {
                    let page = self.controller.memory.rom_page_data_mut(0);
                    asset.read_exact(page)?;
                }
            }
            ZXMachine::Sinclair128K => {
                if let (Some(mut page0_asset), Some(mut page1_asset)) =
                    (self.host.rom(0), self.host.rom(1))
                {
                    let page = self.controller.memory.rom_page_data_mut(0);
                    page0_asset.read_exact(page)?;

                    let page = self.controller.memory.rom_page_data_mut(1);
                    page1_asset.read_exact(page)?;
                }
            }
        };

        Ok(())
    }

    pub fn play_tape(&mut self) {
        self.controller.tape.play();
    }

    pub fn stop_tape(&mut self) {
        self.controller.tape.stop();
    }

    pub fn send_key(&mut self, key: ZXKey, pressed: bool) {
        self.controller.send_key(key, pressed);
    }

    pub fn send_kempston_key(&mut self, key: KempstonKey, state: bool) {
        if let Some(joy) = &mut self.controller.kempston {
            joy.key(key, state);
        }
    }

    pub fn next_audio_sample(&mut self) -> Option<SoundSample<f32>> {
        self.controller.mixer.pop()
    }

    fn process_event(&mut self, event: Event) {
        let Event { kind: e, time: _ } = event;
        match e {
            // Fast tape loading found, use it
            EventKind::FastTapeLoad if self.controller.tape.can_fast_load() && self.fast_load => {
                loaders::tap::fast_load_tap(self);
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
    pub fn emulate_frames<S>(&mut self, max_time: Duration, stopwatch: &mut S) -> Duration
    where
        S: Stopwatch,
    {
        let mut time = Duration::new(0, 0);
        'frame: loop {
            // start of current frame
            stopwatch.reset();
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
                        return stopwatch.measure();
                    };
                // if speed is maximal.
                } else {
                    // if any frame passed then break cpu loop, but try to start new frame
                    if self.controller.frames_count() != 0 {
                        break 'cpu;
                    }
                }
            }
            time += stopwatch.measure();
            // if time is bigger than `max_time` then stop emulation cycle
            if time > max_time {
                break 'frame;
            }
        }
        self.controller.clear_events();
        return time;
    }
}
