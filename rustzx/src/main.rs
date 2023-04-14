//!
//! TODO
//! UNDER CONSTRUCTION
//! Will be refactored

mod app;
mod host;

use app::{Settings, sound::DEFAULT_SAMPLE_RATE};
use structopt::StructOpt;

use winit::{
    event::{Event, WindowEvent, StartCause, DeviceEvent, ScanCode, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::path::Path;
use host::{DetectedFileKind, AppHostContext};

use anyhow::anyhow;
use rustzx_core::zx::{keys::ZXKey, joy::kempston::KempstonKey};
use crate::app::{sound::SoundDevice, SoundBackend};
use crate::app::video::wgpu as wgpu_video;


struct EmulationState {
    emulator: rustzx_core::Emulator<host::AppHost>,
    snd: Option<Box<dyn SoundDevice>>,
}

impl EmulationState {
    /// Starts application itself
    pub fn from_config(settings: Settings) -> anyhow::Result<Self> {
        let snd = if !settings.disable_sound {
            let backend = create_sound_backend(&settings).expect("TODO: handle error properly");
            Some(backend)
        } else {
            None
        };

        let sample_rate = snd
            .as_ref()
            .map(|s| s.sample_rate())
            .unwrap_or(DEFAULT_SAMPLE_RATE);

        let mut emulator = rustzx_core::Emulator::new(settings.to_rustzx_settings(44100), AppHostContext)
            .map_err(|e| anyhow!("Failed to construct emulator: {}", e))?;

        if let Some(rom) = settings.rom.as_ref() {
            emulator
                .load_rom(host::load_rom(rom, settings.machine)?)
                .map_err(|e| anyhow!("Emulator failed to load rom: {}", e))?;
        }
        if let Some(snapshot) = settings.snap.as_ref() {
            emulator
                .load_snapshot(host::load_snapshot(snapshot)?)
                .map_err(|e| anyhow!("Emulator failed to load snapshot: {}", e))?;
        }
        if let Some(tape) = settings.tape.as_ref() {
            emulator
                .load_tape(host::load_tape(tape)?)
                .map_err(|e| anyhow!("Emulator failed to load tape: {}", e))?;
        }
        if let Some(screen) = settings.screen.as_ref() {
            emulator
                .load_screen(host::load_screen(screen)?)
                .map_err(|e| anyhow!("Emulator failed to load screen: {}", e))?;
        }

        let file_autodetect = settings.file_autodetect.clone();

        let mut app = Self {
            emulator,
            snd,
        };

        if let Some(file) = file_autodetect.as_ref() {
            app.load_file_autodetect(file)?;
        }

        Ok(app)
    }

    fn load_file_autodetect(&mut self, path: &Path) -> anyhow::Result<()> {
        match host::detect_file_type(path)? {
            DetectedFileKind::Snapshot => {
                self.emulator
                    .load_snapshot(host::load_snapshot(path)?)
                    .map_err(|e| {
                        anyhow!("Emulator failed to load auto-detected snapshot: {}", e)
                    })?;
            }
            DetectedFileKind::Tape => {
                self.emulator
                    .load_tape(host::load_tape(path)?)
                    .map_err(|e| anyhow!("Emulator failed to load auto-detected tape: {}", e))?;
            }
            DetectedFileKind::Screen => self
                .emulator
                .load_screen(host::load_screen(path)?)
                .map_err(|e| anyhow!("Emulator failed load screen via auto-detect: {}", e))?,
        }
        Ok(())
    }

    fn update(&mut self) -> anyhow::Result<()> {
        self.emulator.emulate_frames(std::time::Duration::from_millis(15))
            .expect("TODO: handle emulator error");

        // if sound enabled sound ganeration allowed then move samples to sound thread
        if let Some(ref mut snd) = self.snd {
            // if can be turned off even on speed change, so check it everytime
            if self.emulator.have_sound() {
                while let Some(sample) = self.emulator.next_audio_sample() {
                    snd.send_sample(sample);
                }
            }
        }

        Ok(())
    }

    fn canvas_buffer(&self) -> &[u8] {
        self.emulator.screen_buffer().data()
    }

    fn screen_buffer(&self) -> &[u8] {
        self.emulator.border_buffer().data()
    }
}

fn main() {
    pollster::block_on(run()).unwrap();
}

async fn run() -> Result<(), anyhow::Error> {
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|level_str| level_str.parse().ok())
        .unwrap_or(log::Level::Warn);

    simple_logger::init_with_level(log_level).expect("Failed to initialize logger");

    let settings = Settings::from_args();

    let mut emulation_state = EmulationState::from_config(settings)?;


    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let params = wgpu_video::ScreenParams {};

    let mut render = wgpu_video::Screen::init(params, &window).await?;

    let mut last_emulated_frame_time = std::time::Instant::now();
    let frame_time = std::time::Duration::from_millis(1000 / rustzx_core::zx::constants::FPS as u64);

    let mut fps_samples = [0f32; 100];
    let mut fps_sample_index = 0;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                render.update_canvas(emulation_state.canvas_buffer());
                render.update_screen(emulation_state.screen_buffer());
                render.render();
            }
            Event::MainEventsCleared => {}
            Event::NewEvents(StartCause::Init) => {
                // Emulate first frame after application start
                emulation_state.update().unwrap();

                // Schedule next frame
                last_emulated_frame_time = std::time::Instant::now();
                control_flow.set_wait_until(last_emulated_frame_time + frame_time);
            }
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                let elapsed = last_emulated_frame_time.elapsed();

                last_emulated_frame_time = std::time::Instant::now();
                control_flow.set_wait_until(last_emulated_frame_time + frame_time);

                fps_samples[fps_sample_index] = elapsed.as_millis() as f32;
                fps_sample_index = (fps_sample_index + 1) % fps_samples.len();

                let elapsed_average = fps_samples.iter().sum::<f32>() / fps_samples.len() as f32;
                window.set_title(&format!("RustZX - {:.2} FPS", 1000.0 / elapsed_average));

                emulation_state.update().unwrap();
            }
            // Handle resize events
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let width = size.width.max(1);
                let height = size.height.max(1);
                render.resize(width, height);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput{
                    input: KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                    ..
                },
                ..
            } => {
                if let Some(zx_key) = map_zx_key(key) {
                    if state == winit::event::ElementState::Pressed {
                        emulation_state.emulator.send_key(zx_key, true);
                    } else {
                        emulation_state.emulator.send_key(zx_key, false);
                    }
                }
                if let Some(key) = map_kempston_key(key) {
                    if state == winit::event::ElementState::Pressed {
                        emulation_state.emulator.send_kempston_key(key, true);
                    } else {
                        emulation_state.emulator.send_kempston_key(key, false);
                    }
                }

            }
            _ => (),
        }
    });

    Ok(())
}


fn map_zx_key(key: winit::event::VirtualKeyCode) -> Option<ZXKey> {
    use winit::event::VirtualKeyCode as Vk;
    match key {
        Vk::A => Some(ZXKey::A),
        Vk::B => Some(ZXKey::B),
        Vk::C => Some(ZXKey::C),
        Vk::D => Some(ZXKey::D),
        Vk::E => Some(ZXKey::E),
        Vk::F => Some(ZXKey::F),
        Vk::G => Some(ZXKey::G),
        Vk::H => Some(ZXKey::H),
        Vk::I => Some(ZXKey::I),
        Vk::J => Some(ZXKey::J),
        Vk::K => Some(ZXKey::K),
        Vk::L => Some(ZXKey::L),
        Vk::M => Some(ZXKey::M),
        Vk::N => Some(ZXKey::N),
        Vk::O => Some(ZXKey::O),
        Vk::P => Some(ZXKey::P),
        Vk::Q => Some(ZXKey::Q),
        Vk::R => Some(ZXKey::R),
        Vk::S => Some(ZXKey::S),
        Vk::T => Some(ZXKey::T),
        Vk::U => Some(ZXKey::U),
        Vk::V => Some(ZXKey::V),
        Vk::W => Some(ZXKey::W),
        Vk::X => Some(ZXKey::X),
        Vk::Y => Some(ZXKey::Y),
        Vk::Z => Some(ZXKey::Z),
        Vk::Key1 => Some(ZXKey::N1),
        Vk::Key2 => Some(ZXKey::N2),
        Vk::Key3 => Some(ZXKey::N3),
        Vk::Key4 => Some(ZXKey::N4),
        Vk::Key5 => Some(ZXKey::N5),
        Vk::Key6 => Some(ZXKey::N6),
        Vk::Key7 => Some(ZXKey::N7),
        Vk::Key8 => Some(ZXKey::N8),
        Vk::Key9 => Some(ZXKey::N9),
        Vk::Key0 => Some(ZXKey::N0),
        Vk::Return => Some(ZXKey::Enter),
        Vk::Space => Some(ZXKey::Space),
        Vk::LShift | Vk::RShift => Some(ZXKey::Shift),
        Vk::LControl | Vk::RControl => Some(ZXKey::SymShift),
        _ => None,
    }
}

fn map_kempston_key(key: winit::event::VirtualKeyCode) -> Option<KempstonKey> {
    use winit::event::VirtualKeyCode as Vk;
    match key {
        Vk::Up => Some(KempstonKey::Up),
        Vk::Down => Some(KempstonKey::Down),
        Vk::Left => Some(KempstonKey::Left),
        Vk::Right => Some(KempstonKey::Right),
        Vk::LAlt | Vk::RAlt => Some(KempstonKey::Fire),
        _ => None,
    }
}

fn create_sound_backend(settings: &Settings) -> anyhow::Result<Box<dyn SoundDevice>> {
    use crate::app::sound;

    let backend: Box<dyn SoundDevice> = match settings.sound_backend {
        SoundBackend::Cpal => Box::new(sound::SoundCpal::new(settings)?),
    };
    Ok(backend)
}
