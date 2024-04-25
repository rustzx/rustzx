//! Platform-independent high-level Emulator interaction module
mod fastload;
pub mod poke;
mod screenshot;
mod snapshot;

use crate::{
    error::RomLoadError,
    host::{
        DataRecorder, Host, LoadableAsset, RomFormat, RomSet, Screen, ScreenAsset, Snapshot,
        SnapshotAsset, SnapshotRecorder, Stopwatch, Tape,
    },
    settings::RustzxSettings,
    utils::EmulationMode,
    zx::{
        controller::ZXController,
        events::EmulationEvents,
        joy::{
            kempston::KempstonKey,
            sinclair::{SinclairJoyNum, SinclairKey},
        },
        keys::{CompoundKey, ZXKey},
        mouse::kempston::{KempstonMouseButton, KempstonMouseWheelDirection},
        tape::{Tap, TapeImpl},
        video::colors::ZXColor,
    },
    Result,
};
use core::time::Duration;
use rustzx_z80::Z80;

#[cfg(feature = "sound")]
use crate::zx::sound::sample::SoundSample;
#[cfg(feature = "autoload")]
use crate::{host::BufferCursor, zx::machine::ZXMachine};

/// Represents emulator stop reason
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EmulationStopReason {
    /// Requested frames count have been emulated successfully
    Completed,
    /// Emulation time limit has been reached
    Timeout,
    /// Emulator has reached breakpoint address
    Breakpoint,
}

/// Represents emulator emulation result
pub struct EmulationInfo {
    /// Emulation duration in emulated time (not real time)
    pub duration: Duration,
    /// Emulation stop reason, see [EmulationStopReason]
    pub stop_reason: EmulationStopReason,
}

/// Represents main Emulator structure
pub struct Emulator<H: Host> {
    settings: RustzxSettings,
    cpu: Z80,
    controller: ZXController<H>,
    mode: EmulationMode,
    fast_load: bool,
    #[cfg(feature = "sound")]
    sound_enabled: bool,
}

impl<H: Host> Emulator<H> {
    /// Constructs new emulator
    /// # Arguments
    /// `settings` - emulator settings
    pub fn new(settings: RustzxSettings, context: H::Context) -> Result<Self> {
        let mode = settings.emulation_mode;
        let fast_load = settings.tape_fastload_enabled;
        #[cfg(feature = "sound")]
        let sound_enabled = settings.sound_enabled;

        let cpu = Z80::default();
        let controller = ZXController::<H>::new(&settings, context);

        let this = Self {
            settings,
            cpu,
            controller,
            mode,
            fast_load,
            #[cfg(feature = "sound")]
            sound_enabled,
        };

        Ok(this)
    }

    /// changes emulation speed
    pub fn set_speed(&mut self, new_speed: EmulationMode) {
        self.mode = new_speed;
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

    /// enables/disables AY
    #[cfg(all(feature = "sound", feature = "ay"))]
    pub fn enable_ay(&mut self, value: bool) {
        self.settings.ay_enabled = value;
    }

    /// function for sound generation request check
    #[cfg(feature = "sound")]
    pub fn have_sound(&self) -> bool {
        // enable sound only if speed is normal
        if let EmulationMode::FrameCount(1) = self.mode {
            self.sound_enabled
        } else {
            false
        }
    }

    pub fn load_snapshot(&mut self, snapshot: Snapshot<impl SnapshotAsset>) -> Result<()> {
        match snapshot {
            Snapshot::Sna(asset) => snapshot::sna::load(self, asset),
            Snapshot::Szx(asset) => snapshot::szx::load(self, asset),
        }
    }

    pub fn save_snapshot<R>(&mut self, recorder: SnapshotRecorder<R>) -> Result<()>
    where
        R: DataRecorder,
    {
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

        #[cfg(feature = "autoload")]
        if self.settings.autoload_enabled {
            let snapshot = match self.settings.machine {
                ZXMachine::Sinclair48K => &snapshot::autoload::tape::SNAPSHOT_SNA_48K,
                ZXMachine::Sinclair128K => &snapshot::autoload::tape::SNAPSHOT_SNA_128K,
            };

            self.load_snapshot(Snapshot::Sna(BufferCursor::new(snapshot)))?;
        }

        Ok(())
    }

    fn load_rom_binary_16k_pages(&mut self, mut rom: impl RomSet) -> Result<()> {
        let page_count = self.settings.machine.specs().rom_pages;

        for page_index in 0..page_count {
            let mut page_asset = rom.next_asset().ok_or(RomLoadError::MoreAssetsRequired)?;
            let page_buffer = self.controller.memory.rom_page_data_mut(page_index);
            page_asset.read_exact(page_buffer)?;
        }

        Ok(())
    }

    pub fn load_rom(&mut self, rom: impl RomSet) -> Result<()> {
        match rom.format() {
            RomFormat::Binary16KPages => self.load_rom_binary_16k_pages(rom),
        }
    }

    pub fn load_screen(&mut self, screen: Screen<impl ScreenAsset>) -> Result<()> {
        match screen {
            Screen::Scr(asset) => screenshot::scr::load(self, asset)?,
        };

        Ok(())
    }

    pub fn play_tape(&mut self) {
        self.controller.tape.play();
    }

    pub fn stop_tape(&mut self) {
        self.controller.tape.stop();
    }

    /// Rewinds tape. May return error if underlying tape asset failed to
    /// perform seek operation to go back to the the beginning of the tape
    pub fn rewind_tape(&mut self) -> Result<()> {
        self.controller.tape.rewind()
    }

    pub fn screen_buffer(&self) -> &H::FrameBuffer {
        self.controller.screen.frame_buffer()
    }

    #[cfg(feature = "precise-border")]
    pub fn border_buffer(&self) -> &H::FrameBuffer {
        self.controller.border.frame_buffer()
    }

    pub fn set_io_extender(&mut self, extender: H::IoExtender) {
        self.controller.io_extender = Some(extender);
    }

    pub fn io_extender(&mut self) -> Option<&mut H::IoExtender> {
        self.controller.io_extender.as_mut()
    }

    /// Sets [Host::DebugInterface] for the emulator instance
    pub fn set_debug_interface(&mut self, debug_interface: H::DebugInterface) {
        self.controller.debug_interface = Some(debug_interface);
    }

    /// Returns current [Host::DebugInterface] instance
    pub fn debug_interface(&mut self) -> Option<&mut H::DebugInterface> {
        self.controller.debug_interface.as_mut()
    }

    /// Reads byte from memory
    pub fn peek(&self, addr: u16) -> u8 {
        self.controller.memory.read(addr)
    }

    pub fn border_color(&self) -> ZXColor {
        self.controller.border_color
    }

    pub fn send_key(&mut self, key: ZXKey, pressed: bool) {
        self.controller.send_key(key, pressed);
    }

    pub fn send_compound_key(&mut self, key: CompoundKey, pressed: bool) {
        self.controller.send_compound_key(key, pressed);
    }

    pub fn send_kempston_key(&mut self, key: KempstonKey, pressed: bool) {
        if let Some(joy) = &mut self.controller.kempston {
            joy.key(key, pressed);
        }
    }

    pub fn send_sinclair_key(&mut self, num: SinclairJoyNum, key: SinclairKey, pressed: bool) {
        self.controller.send_sinclair_key(num, key, pressed);
    }

    pub fn send_mouse_button(&mut self, button: KempstonMouseButton, pressed: bool) {
        self.controller.send_mouse_button(button, pressed);
    }

    pub fn send_mouse_wheel(&mut self, dir: KempstonMouseWheelDirection) {
        self.controller.send_mouse_wheel(dir);
    }

    pub fn send_mouse_pos_diff(&mut self, x: i8, y: i8) {
        self.controller.send_mouse_pos_diff(x, y);
    }

    #[cfg(feature = "sound")]
    pub fn next_audio_sample(&mut self) -> Option<SoundSample<f32>> {
        self.controller.mixer.pop()
    }

    fn process_fast_load_event(&mut self) -> Result<()> {
        if self.controller.tape.can_fast_load() && self.fast_load {
            fastload::tap::fast_load_tap(self)?;
        }
        Ok(())
    }

    /// Execute `poke::Poke` action on the emulator
    pub fn execute_poke(&mut self, poke: impl poke::Poke) {
        for action in poke.actions().iter().copied() {
            match action {
                poke::PokeAction::Mem { addr, value } => {
                    self.controller.memory.force_write(addr, value);
                }
            }
        }
    }

    /// Perform emulatio up to `emulation_limit` duration, returns actual elapsed duration
    pub fn emulate_frames(&mut self, emulation_limit: Duration) -> Result<EmulationInfo> {
        let stopwatch = H::EmulationStopwatch::new();
        // frame loop
        loop {
            // reset controller internal frame counter
            self.controller.reset_frame_counter();
            'cpu: loop {
                // Emulation step. if instant event happened then accept in and execute
                self.cpu.emulate(&mut self.controller);
                if let Some(e) = self.controller.take_last_emulation_error() {
                    return Err(e);
                }

                let events = self.controller.take_events();
                if !events.is_empty() {
                    if events.contains(EmulationEvents::TAPE_FAST_LOAD_TRIGGER_DETECTED) {
                        self.process_fast_load_event()?;
                    }
                    if events.contains(EmulationEvents::PC_BREAKPOINT) {
                        return Ok(EmulationInfo {
                            duration: stopwatch.measure(),
                            stop_reason: EmulationStopReason::Breakpoint,
                        });
                    }
                }

                match self.mode {
                    EmulationMode::FrameCount(frames) => {
                        if self.controller.frames_count() >= frames {
                            return Ok(EmulationInfo {
                                duration: stopwatch.measure(),
                                stop_reason: EmulationStopReason::Completed,
                            });
                        };
                    }
                    EmulationMode::Max => {
                        if self.controller.frames_count() != 0 {
                            break 'cpu;
                        }
                    }
                }
            }
            // if time is bigger than `max_time` then stop emulation cycle
            if stopwatch.measure() > emulation_limit {
                return Ok(EmulationInfo {
                    duration: stopwatch.measure(),
                    stop_reason: EmulationStopReason::Timeout,
                });
            }
        }
    }
}
