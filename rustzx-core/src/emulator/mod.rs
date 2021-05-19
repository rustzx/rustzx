//! Platform-independent high-level Emulator interaction module
mod loaders;
mod snapshot;

use crate::{
    error::RomLoadError,
    host::{Host, LoadableAsset, RomFormat, RomSet, Snapshot, SnapshotRecorder, Tape},
    settings::RustzxSettings,
    utils::EmulationSpeed,
    z80::Z80,
    zx::{
        controller::ZXController,
        events::EmulationEvents,
        joy::kempston::KempstonKey,
        keys::ZXKey,
        tape::{Tap, TapeImpl},
        video::colors::ZXColor,
    },
    Result,
};

#[cfg(feature = "sound")]
use crate::zx::sound::sample::SoundSample;

use core::time::Duration;

/// Represents main Emulator structure
pub struct Emulator<H: Host> {
    settings: RustzxSettings,
    cpu: Z80,
    controller: ZXController<H>,
    speed: EmulationSpeed,
    fast_load: bool,
    #[cfg(feature = "sound")]
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
    pub fn new(settings: RustzxSettings, context: H::Context) -> Result<Self> {
        let speed = settings.emulation_speed;
        let fast_load = settings.tape_fastload;
        #[cfg(feature = "sound")]
        let sound_enabled = settings.sound_enabled;

        let cpu = Z80::default();
        let controller = ZXController::<H>::new(&settings, context);

        let this = Self {
            settings,
            cpu,
            controller,
            speed,
            fast_load,
            #[cfg(feature = "sound")]
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
    #[cfg(feature = "sound")]
    pub fn set_sound(&mut self, value: bool) {
        self.sound_enabled = value;
    }

    /// function for sound generation request check
    #[cfg(feature = "sound")]
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
            Snapshot::Sna(asset) => snapshot::sna::load(self, asset),
        }
    }

    pub fn save_snapshot(&mut self, recorder: SnapshotRecorder<H::SnapshotRecorder>) -> Result<()> {
        match recorder {
            SnapshotRecorder::Sna(recorder) => snapshot::sna::save(self, recorder),
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

    pub fn screen_buffer(&self) -> &H::FrameBuffer {
        self.controller.screen.frame_buffer()
    }

    #[cfg(feature = "precise-border")]
    pub fn border_buffer(&self) -> &H::FrameBuffer {
        self.controller.border.frame_buffer()
    }

    pub fn border_color(&self) -> ZXColor {
        self.controller.border_color
    }

    pub fn send_key(&mut self, key: ZXKey, pressed: bool) {
        self.controller.send_key(key, pressed);
    }

    pub fn send_kempston_key(&mut self, key: KempstonKey, state: bool) {
        if let Some(joy) = &mut self.controller.kempston {
            joy.key(key, state);
        }
    }

    #[cfg(feature = "sound")]
    pub fn next_audio_sample(&mut self) -> Option<SoundSample<f32>> {
        self.controller.mixer.pop()
    }

    fn process_events(&mut self, event: EmulationEvents) -> Result<()> {
        if event.contains(EmulationEvents::TAPE_FAST_LOAD_TRIGGER_DETECTED)
            && self.controller.tape.can_fast_load()
            && self.fast_load
        {
            loaders::tap::fast_load_tap(self)?;
        }
        Ok(())
    }

    /// Emulate frames, maximum in `max_time` time, returns emulation time in nanoseconds
    /// in most cases time is max 1/50 of second, even when using
    /// loader acceleration
    pub fn emulate_frames<S>(&mut self, max_time: Duration, stopwatch: &mut S) -> Result<Duration>
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
                self.cpu.emulate(&mut self.controller);
                if let Some(e) = self.controller.take_last_emulation_error() {
                    return Err(e);
                }
                if !self.controller.events().is_empty() {
                    self.process_events(self.controller.events())?;
                    self.controller.clear_events();
                }
                // If speed is defined
                if let EmulationSpeed::Definite(multiplier) = self.speed {
                    if self.controller.frames_count() >= multiplier {
                        // no more frames
                        return Ok(stopwatch.measure());
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
        Ok(time)
    }
}
