use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use rustzx_core::{
    host::{BufferCursor, FrameBuffer, FrameBufferSource, Host, HostContext, Tape},
    zx::{
        machine::ZXMachine,
        sound::ay::ZXAYMode,
        video::colors::{ZXBrightness, ZXColor},
    },
    EmulationMode, Emulator, RustzxSettings, Stopwatch,
};

// TODO(WIP): Move to new `rustzx-utils` crate?
// TODO(WIP): Add Stopwatch associated type to host instead?
struct InstantStopwatch {
    timestamp: Instant,
}

impl Default for InstantStopwatch {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
        }
    }
}

impl Stopwatch for InstantStopwatch {
    fn reset(&mut self) {
        self.timestamp = Instant::now();
    }

    fn measure(&self) -> Duration {
        self.timestamp.elapsed()
    }
}

fn make_png_palette() -> Vec<u8> {
    // TODO(WIP): move `Palette` form rustzx to new `rustzx-utils` crate
    // and get default color pallete from there?
    const COLORS: [[u8; 4]; 16] = [
        // normal
        0x000000FF_u32.to_be_bytes(),
        0x0000CDFF_u32.to_be_bytes(),
        0xCD0000FF_u32.to_be_bytes(),
        0xCD00CDFF_u32.to_be_bytes(),
        0x00CD00FF_u32.to_be_bytes(),
        0x00CDCDFF_u32.to_be_bytes(),
        0xCDCD00FF_u32.to_be_bytes(),
        0xCDCDCDFF_u32.to_be_bytes(),
        // bright
        0x000000FF_u32.to_be_bytes(),
        0x0000FFFF_u32.to_be_bytes(),
        0xFF0000FF_u32.to_be_bytes(),
        0xFF00FFFF_u32.to_be_bytes(),
        0x00FF00FF_u32.to_be_bytes(),
        0x00FFFFFF_u32.to_be_bytes(),
        0xFFFF00FF_u32.to_be_bytes(),
        0xFFFFFFFF_u32.to_be_bytes(),
    ];

    COLORS
        .iter()
        .fold(Vec::with_capacity(4 * 16), |mut buffer, color| {
            buffer.extend_from_slice(&color[0..3]);
            buffer
        })
}

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
        let mask = 0xF0 >> (pixel_index % 2) * 4;
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

struct TesterHost;

impl Host for TesterHost {
    type Context = TesterContext;
    type FrameBuffer = FrameContent;
    type TapeAsset = BufferCursor<Vec<u8>>;
}

pub struct RustZXTester {
    emulator: Emulator<TesterHost>,
    test_name: String,
}

pub mod presets {
    use super::*;

    pub fn settings_48k_nosound() -> RustzxSettings {
        RustzxSettings {
            machine: ZXMachine::Sinclair48K,
            emulation_mode: EmulationMode::FrameCount(1),
            tape_fastload_enabled: true,
            kempston_enabled: true,
            mouse_enabled: true,
            ay_mode: ZXAYMode::ABC,
            ay_enabled: false,
            beeper_enabled: false,
            sound_enabled: false,
            sound_volume: 100,
            sound_sample_rate: 44100,
            load_default_rom: true,
            autoload_enabled: true,
        }
    }
}

impl RustZXTester {
    pub fn new(test_name: &str, settings: RustzxSettings) -> Self {
        let emulator = Emulator::new(settings, TesterContext::default())
            .expect("Failed to initialize emulator");

        Self {
            emulator,
            test_name: test_name.to_owned(),
        }
    }

    fn assets_folder(&self) -> PathBuf {
        Path::new("test_data/asset").to_owned()
    }

    fn expected_data_folder(&self) -> PathBuf {
        Path::new("test_data/expected").join(&self.test_name)
    }

    fn actual_data_folder(&self) -> PathBuf {
        Path::new("test_data/actual").join(&self.test_name)
    }

    pub fn load_tape(&mut self, name: impl AsRef<Path>) {
        let path = self.assets_folder().join(name);
        let content = std::fs::read(path).expect("Failed to read test tape file");
        self.emulator
            .load_tape(Tape::Tap(BufferCursor::new(content)))
            .expect("Failed to load test tape");
    }

    fn get_screen(&self) -> Vec<u8> {
        self.emulator.screen_buffer().to_png()
    }

    fn get_border(&self) -> Vec<u8> {
        self.emulator.border_buffer().to_png()
    }

    pub fn emulate_for(&mut self, duration: Duration) {
        const FRAME_HOST_DURATION_LIMIT: Duration = Duration::from_millis(100);
        const FRAME_EMULATED_DURATION: Duration = Duration::from_millis(20);

        let mut emulated_duration = Duration::from_secs(0);
        while emulated_duration < duration {
            let mut stopwatch = InstantStopwatch::default();
            self.emulator
                .emulate_frames(FRAME_HOST_DURATION_LIMIT, &mut stopwatch)
                .expect("Emulation failed");
            emulated_duration += FRAME_EMULATED_DURATION;
        }
    }

    pub fn compare_buffer_with_file(&self, actual: Vec<u8>, name: impl AsRef<Path>) {
        let path = name.as_ref();
        let expected = std::fs::read(self.expected_data_folder().join(&path)).unwrap_or_default();

        if actual != expected {
            eprintln!("Integration test failed, writing actual data...");
            // Wirte actual output for further investigation
            std::fs::create_dir_all(self.actual_data_folder())
                .expect("Failed to create expected data dir");
            std::fs::write(self.actual_data_folder().join(path), actual)
                .expect("Failed to write actual data");

            panic!(
                "Comparison with {} failed; Actual data has been saved for further investigation",
                path.display()
            );
        }
    }

    pub fn expect_screen(&self, name: impl AsRef<Path>) {
        self.compare_buffer_with_file(self.get_screen(), name);
    }

    pub fn expect_border(&self, name: impl AsRef<Path>) {
        self.compare_buffer_with_file(self.get_border(), name);
    }
}
