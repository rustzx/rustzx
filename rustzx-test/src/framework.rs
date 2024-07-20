use expect_test::Expect;
use rustzx_core::{
    host::{
        BufferCursor, DebugInterface, FrameBuffer, FrameBufferSource, Host, HostContext,
        IoExtender, RomFormat, RomSet, Snapshot, Tape,
    },
    poke,
    zx::{
        keys::ZXKey,
        machine::ZXMachine,
        sound::ay::ZXAYMode,
        video::colors::{ZXBrightness, ZXColor},
    },
    EmulationMode, EmulationStopReason, Emulator, RustzxSettings,
};
use rustzx_utils::{
    io::{DynamicAsset, GzipAsset},
    palette::rgba::ORIGINAL as DEFAULT_PALETTE,
    stopwatch::InstantStopwatch,
};
use std::{
    collections::VecDeque,
    io::Cursor,
    path::{Path, PathBuf},
    time::Duration,
};

const DEFAULT_SOUND_BITRATE: usize = 44100;
const FRAME_HOST_DURATION_LIMIT: Duration = Duration::from_millis(100);
const FRAME_EMULATED_DURATION: Duration = Duration::from_millis(20);
const DEFAULT_SYNC_TIMEOUT: Duration = Duration::from_secs(3);

// TODO(#83): Add tests for gigascreen

struct FrameContent {
    buffer: Vec<u8>,
    width: usize,
    height: usize,
}

impl FrameBuffer for FrameContent {
    type Context = TesterFrameBufferContext;

    fn new(
        width: usize,
        height: usize,
        _source: FrameBufferSource,
        context: Self::Context,
    ) -> Self {
        if context.use_gigascreen {
            unimplemented!("Gigascreen tests are not yet implemented");
        } else {
            let buffer_size = (width * height) / 2;
            Self {
                buffer: vec![0u8; buffer_size],
                width,
                height,
            }
        }
    }

    fn set_color(&mut self, x: usize, y: usize, color: ZXColor, brightness: ZXBrightness) {
        let pixel_index = x + y * self.width;
        let buffer_index = pixel_index / 2;
        // 0xF0 mask for even pixels, 0x0F mask for odd pixels
        let mask = 0xF0 >> ((pixel_index % 2) * 4);
        let indexed_color = (color as u8) + (brightness as u8) * 8;
        // 0x0A => 0xAA, 0x03  => 0x33, etc.
        let color_overlay_byte = indexed_color | (indexed_color << 4);
        // clear previous color nibble and set to new value
        self.buffer[buffer_index] =
            (self.buffer[buffer_index] & (!mask)) | (color_overlay_byte & mask)
    }
}

impl FrameContent {
    pub fn to_png(&self) -> Vec<u8> {
        let mut out = vec![];

        {
            let mut encoder = png::Encoder::new(&mut out, self.width as u32, self.height as u32);
            encoder.set_depth(png::BitDepth::Four);
            encoder.set_color(png::ColorType::Indexed);
            encoder.set_palette(make_png_palette());
            let mut writer = encoder.write_header().expect("Failed to write PNG header");
            writer
                .write_image_data(&self.buffer)
                .expect("Failed to write PNG data");
        }

        out
    }
}

#[derive(Clone)]
struct TesterFrameBufferContext {
    use_gigascreen: bool,
}

#[derive(Default)]
struct TesterContext;

impl HostContext<TesterHost> for TesterContext {
    fn frame_buffer_context(&self) -> <FrameContent as FrameBuffer>::Context {
        TesterFrameBufferContext {
            use_gigascreen: false,
        }
    }
}

#[derive(Default)]
pub struct DebugPort {
    stdin: VecDeque<u8>,
    stdout: VecDeque<u8>,
}

impl IoExtender for DebugPort {
    fn write(&mut self, _: u16, data: u8) {
        self.stdout.push_back(data);
    }

    fn read(&mut self, _: u16) -> u8 {
        self.stdin.pop_front().unwrap_or(0)
    }

    fn extends_port(&self, port: u16) -> bool {
        port == 0xCCCC
    }
}

impl DebugPort {
    pub fn put_byte(&mut self, b: u8) {
        self.stdin.push_back(b);
    }

    pub fn take_byte(&mut self) -> Option<u8> {
        self.stdout.pop_front()
    }

    pub fn put_text(&mut self, s: &str) {
        self.stdin.extend(s.as_bytes())
    }

    pub fn take_text(&mut self) -> String {
        let s = Vec::from(std::mem::take(&mut self.stdout));
        String::from_utf8(s).expect("Invalid debug port stdout")
    }

    pub fn take_buffer(&mut self) -> Vec<u8> {
        Vec::from(std::mem::take(&mut self.stdout))
    }

    pub fn reset(&mut self) {
        self.stdin.clear();
        self.stdout.clear();
    }
}

/// A simple debug interface that allows to set breakpoints and check if they were hit.
#[derive(Default)]
struct TestDebugInterface {
    breakpoints: std::collections::HashSet<u16>,
    last_hit: Option<u16>,
}

impl TestDebugInterface {
    pub fn add_breakpoint(&mut self, address: u16) {
        self.breakpoints.insert(address);
    }

    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
        self.last_hit = None;
    }

    pub fn last_breakpoint_hit(&self) -> Option<u16> {
        self.last_hit
    }
}

impl DebugInterface for TestDebugInterface {
    fn check_pc_breakpoint(&mut self, addr: u16) -> bool {
        if self.breakpoints.contains(&addr) {
            self.last_hit = Some(addr);
            return true;
        }
        false
    }
}

struct TesterHost;

impl Host for TesterHost {
    type Context = TesterContext;
    type DebugInterface = TestDebugInterface;
    type EmulationStopwatch = InstantStopwatch;
    type FrameBuffer = FrameContent;
    type IoExtender = DebugPort;
    type TapeAsset = DynamicAsset;
}

pub struct RustZXTester {
    emulator: Emulator<TesterHost>,
    sound_buffer: Option<Vec<i16>>,
    test_name: String,
    sync_timeout: Duration,
}

pub mod presets {
    use super::*;

    pub fn settings_48k_nosound() -> RustzxSettings {
        RustzxSettings {
            machine: ZXMachine::Sinclair48K,
            emulation_mode: EmulationMode::FrameCount(1),
            tape_fastload_enabled: true,
            kempston_enabled: false,
            mouse_enabled: false,
            ay_mode: ZXAYMode::ABC,
            ay_enabled: false,
            beeper_enabled: false,
            sound_enabled: false,
            sound_volume: 100,
            sound_sample_rate: DEFAULT_SOUND_BITRATE,
            load_default_rom: true,
            autoload_enabled: true,
        }
    }

    pub fn settings_128k_nosound() -> RustzxSettings {
        RustzxSettings {
            machine: ZXMachine::Sinclair128K,
            ..settings_48k_nosound()
        }
    }

    pub fn settings_48k() -> RustzxSettings {
        RustzxSettings {
            sound_enabled: true,
            ay_enabled: true,
            beeper_enabled: true,
            ..settings_48k_nosound()
        }
    }

    pub fn settings_128k() -> RustzxSettings {
        RustzxSettings {
            machine: ZXMachine::Sinclair128K,
            ..settings_48k()
        }
    }
}

impl RustZXTester {
    pub fn new(test_name: &str, settings: RustzxSettings) -> Self {
        let emulator =
            Emulator::new(settings, TesterContext).expect("Failed to initialize emulator");

        Self {
            emulator,
            test_name: test_name.to_owned(),
            sound_buffer: None,
            sync_timeout: DEFAULT_SYNC_TIMEOUT,
        }
    }

    fn assets_folder(&self) -> PathBuf {
        Path::new("test_data").to_owned()
    }

    fn actual_data_folder(&self) -> PathBuf {
        Path::new("test_data/actual").join(&self.test_name)
    }

    fn load_asset_data(&mut self, name: impl AsRef<Path>) -> Vec<u8> {
        let path = self.assets_folder().join(name);
        let content = std::fs::read(&path).expect("Failed to load asset");

        if path
            .extension()
            .map(|e| e.to_str().unwrap() == "gz")
            .unwrap_or_default()
        {
            GzipAsset::new(Cursor::new(content))
                .expect("Failed to decompress gz")
                .into_vec()
        } else {
            content
        }
    }

    fn load_asset(&mut self, name: impl AsRef<Path>) -> DynamicAsset {
        BufferCursor::new(self.load_asset_data(name)).into()
    }

    pub fn load_tap(&mut self, name: impl AsRef<Path>) {
        let asset = self.load_asset(name);
        self.emulator
            .load_tape(Tape::Tap(asset))
            .expect("Failed to load test TAP");
    }

    pub fn load_sna(&mut self, name: impl AsRef<Path>) {
        let asset = self.load_asset(name);
        self.emulator
            .load_snapshot(Snapshot::Sna(asset))
            .expect("Failed to load test SNA")
    }

    pub fn load_szx(&mut self, name: impl AsRef<Path>) {
        let asset = self.load_asset(name);
        self.emulator
            .load_snapshot(Snapshot::Szx(asset))
            .expect("Failed to load test SZX")
    }

    pub fn load_single_page_rom(&mut self, name: impl AsRef<Path>) {
        let rom_data = self.load_asset_data(name);
        struct DiagRomSet {
            pages: VecDeque<Vec<u8>>,
        }

        impl RomSet for DiagRomSet {
            type Asset = BufferCursor<Vec<u8>>;

            fn format(&self) -> RomFormat {
                RomFormat::Binary16KPages
            }

            fn next_asset(&mut self) -> Option<Self::Asset> {
                Some(BufferCursor::new(self.pages.pop_front().unwrap()))
            }
        }

        let rom_set = DiagRomSet {
            pages: VecDeque::from(vec![rom_data, vec![0u8; 16 * 1024]]),
        };

        self.emulator.load_rom(rom_set).unwrap();
    }

    fn get_screen(&self) -> Vec<u8> {
        self.emulator.screen_buffer().to_png()
    }

    fn get_border(&self) -> Vec<u8> {
        self.emulator.border_buffer().to_png()
    }

    fn update_sound(&mut self) {
        if let Some(sound_buffer) = &mut self.sound_buffer {
            while let Some(sample) = self.emulator.next_audio_sample() {
                let normalize = |s| (s * i16::MAX as f32) as i16;
                sound_buffer.push(normalize(sample.left));
                sound_buffer.push(normalize(sample.right));
            }
        }
    }

    /// Sets the breakpoint and emulates until it is hit or the timeout is reached
    pub fn emulate_until_breakpoint(&mut self, breakpoint_addr: u16, timeout: Duration) {
        self.clear_breakpoints();
        self.add_breakpoint(breakpoint_addr);

        let result = self.emulate_for(timeout);
        if let EmulationStopReason::Breakpoint = result {
            // Check if last breakpoint is the one we requested
            let last_breakpoint = self.last_breakpoint();
            assert_eq!(
                last_breakpoint, breakpoint_addr,
                "Emulator stopped at breakpoint {:04X}, expected {:04X}",
                last_breakpoint, breakpoint_addr
            );

            return;
        }
        panic!("Emulator failed to hit breakpoint before reaching timeout");
    }

    ///  Emulates for the given duration or until the breakpoint is hit
    pub fn emulate_for(&mut self, duration: Duration) -> EmulationStopReason {
        let mut emulated_duration = Duration::from_secs(0);
        while emulated_duration < duration {
            let result = self
                .emulator
                .emulate_frames(FRAME_HOST_DURATION_LIMIT)
                .expect("Emulation failed");

            self.update_sound();
            emulated_duration += FRAME_EMULATED_DURATION;

            if result.stop_reason == EmulationStopReason::Breakpoint {
                eprintln!(
                    "Requested {} duration, emulated for {}",
                    duration.as_millis(),
                    emulated_duration.as_millis()
                );

                return result.stop_reason;
            }
        }
        EmulationStopReason::Completed
    }

    pub fn emulate_frame(&mut self) {
        self.emulate_for(FRAME_EMULATED_DURATION);
    }

    pub fn last_breakpoint(&mut self) -> u16 {
        self.emulator
            .debug_interface()
            .expect("no breakpoints were set")
            .last_breakpoint_hit()
            .expect("No breapoints were triggered")
    }

    pub fn compare_buffer_with_file(
        &self,
        actual: Vec<u8>,
        name: impl AsRef<Path>,
        expect: Expect,
    ) {
        if TestEnv::save_test_data_enabled() {
            self.save_actual_data(&actual, name.as_ref());
        }

        expect.assert_eq(&actual.fingerprint());
    }

    fn save_actual_data(&self, actual: &[u8], filename: &Path) {
        let filename = self.actual_data_folder().join(filename);
        eprintln!("Saving actual test data file {}", filename.display());

        std::fs::create_dir_all(self.actual_data_folder())
            .expect("Failed to create actual data dir");
        std::fs::write(filename, actual).expect("Failed to write actual data");
    }

    pub fn expect_screen(&self, name: impl AsRef<Path>, expect: Expect) {
        self.compare_buffer_with_file(self.get_screen(), make_screen_filename(name), expect);
    }

    pub fn expect_border(&self, name: impl AsRef<Path>, expect: Expect) {
        self.compare_buffer_with_file(self.get_border(), make_border_filename(name), expect);
    }

    pub fn expect_text(&self, name: impl AsRef<Path>, text: String, expect: Expect) {
        self.compare_buffer_with_file(text.into_bytes(), make_text_filename(name), expect);
    }

    pub fn emulator(&mut self) -> &mut Emulator<impl Host> {
        &mut self.emulator
    }

    pub fn send_keypress(&mut self, key: ZXKey) {
        self.emulator.send_key(key, true);
    }

    pub fn send_keystrokes(&mut self, keystrokes: &[&[ZXKey]], keystroke_delay: Duration) {
        let mut first = true;
        for keys in keystrokes {
            if !first {
                self.emulate_for(keystroke_delay);
            }
            first = false;

            for key in *keys {
                self.emulator.send_key(*key, true);
            }

            self.emulate_for(keystroke_delay);

            for key in *keys {
                self.emulator.send_key(*key, false);
            }
        }
    }

    pub fn start_sound_capture(&mut self) {
        // Pre-allocate 1Mb of memory
        self.sound_buffer.replace(Vec::with_capacity(1024 * 1024));
    }

    pub fn expect_sound(&mut self, name: impl AsRef<Path>, expect: Expect) {
        let data = self
            .sound_buffer
            .take()
            .expect("Sound is not being recorded");

        let mut wav_data = std::io::Cursor::new(vec![]);
        let wav_header = wav::Header::new(
            wav::header::WAV_FORMAT_PCM,
            2,
            DEFAULT_SOUND_BITRATE as u32,
            16,
        );
        wav::write(wav_header, &wav::BitDepth::Sixteen(data), &mut wav_data)
            .expect("Failed to generate wav");

        self.compare_buffer_with_file(wav_data.into_inner(), make_sound_filename(name), expect);
    }

    pub fn enable_debug_port(&mut self) {
        self.emulator.set_io_extender(DebugPort::default());
    }

    pub fn debug_port(&mut self) -> &mut DebugPort {
        self.emulator
            .io_extender()
            .expect("Debug port is not enabled for the current test")
    }

    pub fn sync_target(&mut self) {
        if !self.debug_port().stdout.is_empty() || !self.debug_port().stdin.is_empty() {
            panic!(
                "ERROR: Test may be incorrect, there were some unprocessed data in the target's \
                 port before sync"
            );
        }

        // Host (this test executable) writes and then reads port,
        // while target (emulated snapshot) reads and then writes,
        // this allows to sync both sides and doesn not produce any
        // deadlocks
        self.debug_port().put_byte(1);

        let mut sync_duration = Duration::default();
        loop {
            self.emulate_for(FRAME_EMULATED_DURATION);
            sync_duration += FRAME_EMULATED_DURATION;

            if sync_duration > self.sync_timeout {
                panic!("Timeout reached when trying to sync host with target");
            }

            // Try to consume incoming signal and finish sync
            if self.debug_port().take_byte().is_some() {
                break;
            }
        }
    }

    pub fn set_sync_timeout(&mut self, timeout: Duration) {
        self.sync_timeout = timeout;
    }

    /// Disables the message and key press prompt after a few lines of scroll in BASIC
    pub fn disable_scroll_message(&mut self) {
        self.emulator.execute_poke(poke::DisableScrollMessageRom48);
    }

    pub fn add_breakpoint(&mut self, address: u16) {
        if let Some(interface) = self.emulator.debug_interface() {
            interface.add_breakpoint(address);
        } else {
            let mut interface = TestDebugInterface::default();
            interface.add_breakpoint(address);
            self.emulator.set_debug_interface(interface);
        }
    }

    pub fn clear_breakpoints(&mut self) {
        if let Some(interface) = self.emulator.debug_interface() {
            interface.clear_breakpoints();
        }
    }

    pub fn peek(&mut self, addr: u16) -> u8 {
        self.emulator.peek(addr)
    }
}

struct TestEnv;

impl TestEnv {
    fn save_test_data_enabled() -> bool {
        cfg!(feature = "save-test-data")
    }
}

fn make_png_palette() -> Vec<u8> {
    DEFAULT_PALETTE
        .iter()
        .fold(Vec::with_capacity(4 * 16), |mut buffer, color| {
            buffer.extend_from_slice(&color[0..3]);
            buffer
        })
}

fn make_screen_filename(name: impl AsRef<Path>) -> PathBuf {
    name.as_ref().with_extension("screen.png")
}

fn make_border_filename(name: impl AsRef<Path>) -> PathBuf {
    name.as_ref().with_extension("border.png")
}

fn make_sound_filename(name: impl AsRef<Path>) -> PathBuf {
    name.as_ref().with_extension("wav")
}

fn make_text_filename(name: impl AsRef<Path>) -> PathBuf {
    name.as_ref().with_extension("txt")
}

trait Fingerprintable {
    fn fingerprint(&self) -> String;
}

impl Fingerprintable for Vec<u8> {
    fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::default();
        hasher.update(self);
        let hash = hasher.finalize();
        base64::encode(hash)
    }
}
