//! Platform-independent high-level Emulator interaction module
mod loaders;

use crate::{
    error::RomLoadError,
    host::{Host, LoadableAsset, RomFormat, RomSet, Snapshot, Tape},
    settings::RustzxSettings,
    utils::*,
    z80::*,
    zx::{
        joy::kempston::KempstonKey,
        sound::sample::SoundSample,
        tape::{Tap, TapeImpl},
        ZXController, ZXKey,
    },
    Result,
};

use core::time::Duration;

/// Represents main Emulator structure
pub struct Emulator<H: Host> {
    settings: RustzxSettings,
    cpu: Z80,
    // TODO(#52): eliminate direct access to the controller
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
    pub fn new(settings: RustzxSettings) -> Result<Self> {
        let speed = settings.emulation_speed;
        let fast_load = settings.tape_fastload;
        let sound_enabled = settings.sound_enabled;

        let cpu = Z80::default();
        let controller = ZXController::<H>::new(&settings);

        let this = Self {
            settings,
            cpu,
            controller,
            speed,
            fast_load,
            sound_enabled,
        };

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
            self.sound_enabled
        } else {
            false
        }
    }

    pub fn load_snapshot(&mut self, snapshot: Snapshot<H::SnapshotAsset>) -> Result<()> {
        match snapshot {
            Snapshot::Sna(asset) => loaders::sna::load_sna(self, asset),
        }
    }

    pub fn load_tape(&mut self, tape: Tape<H::TapeAsset>) -> Result<()> {
        match tape {
            Tape::Tap(asset) => {
                self.controller.tape = Tap::from_asset(asset)?.into();
            }
        }

        Ok(())
    }

    fn load_rom_binary_16k_pages(&mut self, mut rom: H::RomSet) -> Result<()> {
        let page_count = self.settings.machine.specs().rom_pages;

        for page_index in 0..page_count {
            let mut page_asset = rom.next_asset().ok_or(RomLoadError::MoreAssetsRequired)?;
            let page_buffer = self.controller.memory.rom_page_data_mut(page_index);
            page_asset.read_exact(page_buffer)?;
        }

        Ok(())
    }

    pub fn load_rom(&mut self, rom: H::RomSet) -> Result<()> {
        match rom.format() {
            RomFormat::Binary16KPages => self.load_rom_binary_16k_pages(rom),
        }
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
        while let Some(event) = self.controller.pop_event() {
            self.process_event(event);
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
        time
    }
}
