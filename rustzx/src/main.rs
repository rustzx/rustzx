//!
//! RustZX application entry point.
//!
//! Handles window creation, the winit event loop, keyboard/mouse input mapping
//! and drives the emulator + wgpu-based renderer.

mod app;
mod host;

use app::{sound::DEFAULT_SAMPLE_RATE, Settings};
use structopt::StructOpt;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use winit::{
    application::ApplicationHandler,
    event::{
        DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, StartCause, WindowEvent,
    },
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use anyhow::anyhow;
use host::{AppHostContext, DetectedFileKind};
use rustzx_core::{
    host::SnapshotRecorder,
    zx::{
        constants::{FPS, SCREEN_HEIGHT, SCREEN_WIDTH},
        joy::{
            kempston::KempstonKey,
            sinclair::{SinclairJoyNum, SinclairKey},
        },
        keys::{CompoundKey, ZXKey},
        mouse::kempston::{KempstonMouseButton, KempstonMouseWheelDirection},
    },
    EmulationMode,
};
use rustzx_utils::io::FileAsset;

use crate::app::sound::SoundDevice;
use crate::app::video::wgpu as wgpu_video;

/// State that owns the emulator core and everything not tied to the window/renderer
struct EmulationState {
    emulator: rustzx_core::Emulator<host::AppHost>,
    snd: Option<Box<dyn SoundDevice>>,
    settings: Settings,
}

impl EmulationState {
    /// Starts application itself
    pub fn from_config(settings: Settings) -> anyhow::Result<Self> {
        let snd = if !settings.disable_sound {
            let backend = create_sound_backend(&settings)
                .map_err(|e| anyhow!("Failed to initialize sound subsystem: {}", e))?;
            Some(backend)
        } else {
            None
        };

        let sample_rate = snd
            .as_ref()
            .map(|s| s.sample_rate())
            .unwrap_or(DEFAULT_SAMPLE_RATE);

        let mut emulator =
            rustzx_core::Emulator::new(settings.to_rustzx_settings(sample_rate), AppHostContext)
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

        let mut state = Self {
            emulator,
            snd,
            settings,
        };

        if let Some(file) = file_autodetect.as_ref() {
            state.load_file_autodetect(file)?;
        }

        Ok(state)
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
        self.emulator
            .emulate_frames(std::time::Duration::from_millis(15))
            .map_err(|e| anyhow!("Emulation step failed: {:#?}", e))?;

        // if sound enabled sound generation allowed then move samples to sound thread
        if let Some(ref mut snd) = self.snd {
            // can be turned off even on speed change, so check it every time
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

    fn border_buffer(&self) -> &[u8] {
        self.emulator.border_buffer().data()
    }

    fn quick_save(&mut self) -> anyhow::Result<()> {
        let new_path = self.last_quick_snapshot_path();
        let prev_path = self.prev_quick_snapshot_path();

        if new_path.exists() {
            if prev_path.exists() {
                fs::remove_file(&prev_path)?;
            }
            fs::rename(&new_path, &prev_path)?;
        }

        let recorder = SnapshotRecorder::Sna(FileAsset::from(fs::File::create(new_path)?));
        self.emulator
            .save_snapshot(recorder)
            .map_err(|e| anyhow!("Failed to save quick snapshot: {}", e))?;
        Ok(())
    }

    fn quick_load(&mut self) -> anyhow::Result<()> {
        let last_snapshot_path = self.last_quick_snapshot_path();
        if !last_snapshot_path.exists() {
            log::warn!("Quick snapshot was not found");
            return Ok(());
        }
        self.emulator
            .load_snapshot(host::load_snapshot(&last_snapshot_path)?)
            .map_err(|e| anyhow!("Emulator failed to load quick snapshot: {}", e))?;
        Ok(())
    }

    fn last_quick_snapshot_path(&self) -> PathBuf {
        if let Some(path) = self.settings.file_autodetect.as_ref() {
            return path.with_extension("rustzx.last.sna");
        }
        Path::new("default.rustzx.last.sna").to_owned()
    }

    fn prev_quick_snapshot_path(&self) -> PathBuf {
        if let Some(path) = self.settings.file_autodetect.as_ref() {
            return path.with_extension("rustzx.prev.sna");
        }
        Path::new("default.rustzx.prev.sna").to_owned()
    }
}

fn create_sound_backend(settings: &Settings) -> anyhow::Result<Box<dyn SoundDevice>> {
    use crate::app::{sound, SoundBackend};

    let backend: Box<dyn SoundDevice> = match settings.sound_backend {
        SoundBackend::Cpal => Box::new(sound::SoundCpal::new(settings)?),
    };
    Ok(backend)
}

/// Renderer + window, only valid while the application is resumed
struct Graphics {
    window: Arc<Window>,
    screen: wgpu_video::Screen,
}

struct App {
    emulation: EmulationState,
    graphics: Option<Graphics>,

    last_emulated_frame_time: Instant,
    frame_time: Duration,

    fps_samples: [f32; 100],
    fps_sample_index: usize,

    kempston_enabled: bool,
    mouse_enabled: bool,
    mouse_sensitivity: usize,
    mouse_locked: bool,
    mouse_x_counter: i32,
    mouse_y_counter: i32,

    enable_joy_keyboard_layer: bool,
    enable_frame_trace: bool,
}

impl App {
    fn new(emulation: EmulationState) -> Self {
        let kempston_enabled = !emulation.settings.disable_kempston;
        let mouse_enabled = emulation.settings.enable_mouse;
        let mouse_sensitivity = emulation.settings.mouse_sensitivity;

        Self {
            emulation,
            graphics: None,
            last_emulated_frame_time: Instant::now(),
            frame_time: Duration::from_millis(1000 / FPS as u64),
            fps_samples: [0f32; 100],
            fps_sample_index: 0,
            kempston_enabled,
            mouse_enabled,
            mouse_sensitivity,
            mouse_locked: false,
            mouse_x_counter: 0,
            mouse_y_counter: 0,
            enable_joy_keyboard_layer: false,
            enable_frame_trace: cfg!(debug_assertions),
        }
    }

    fn window_title(&self) -> String {
        let mut title = format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        if self.enable_joy_keyboard_layer {
            title.push_str(" [JOY]");
        }

        if self.enable_frame_trace {
            let elapsed_average =
                self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32;
            if elapsed_average > 0.0 {
                title.push_str(&format!(" [{:.1} FPS]", 1000.0 / elapsed_average));
            }
            title.push_str(" [FRAME_TRACE]");
        }

        title
    }

    fn update_window_title(&self) {
        if let Some(graphics) = self.graphics.as_ref() {
            graphics.window.set_title(&self.window_title());
        }
    }

    fn lock_mouse(&mut self) {
        if !self.mouse_enabled || self.mouse_locked {
            return;
        }
        if let Some(graphics) = self.graphics.as_ref() {
            if graphics
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .or_else(|_| {
                    graphics
                        .window
                        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                })
                .is_ok()
            {
                graphics.window.set_cursor_visible(false);
                self.mouse_locked = true;
            }
        }
    }

    fn unlock_mouse(&mut self) {
        if !self.mouse_locked {
            return;
        }
        if let Some(graphics) = self.graphics.as_ref() {
            let _ = graphics
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::None);
            graphics.window.set_cursor_visible(true);
        }
        self.mouse_locked = false;
    }

    fn emulate_and_reschedule(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = self.emulation.update() {
            log::error!("Emulation error: {:#}", e);
            event_loop.exit();
            return;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_emulated_frame_time);
        self.last_emulated_frame_time = now;

        self.fps_samples[self.fps_sample_index] = elapsed.as_millis() as f32;
        self.fps_sample_index = (self.fps_sample_index + 1) % self.fps_samples.len();

        if self.enable_frame_trace {
            self.update_window_title();
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(now + self.frame_time));
    }

    fn handle_key_action(&mut self, code: KeyCode, pressed: bool, repeat: bool) {
        // One-shot function keys, only handled on key-down (not on OS auto-repeat)
        if pressed && !repeat {
            match code {
                KeyCode::F1 => {
                    if let Err(e) = self.emulation.quick_save() {
                        log::error!("Quick save failed: {:#}", e);
                    }
                }
                KeyCode::F2 => {
                    if let Err(e) = self.emulation.quick_load() {
                        log::error!("Quick load failed: {:#}", e);
                    }
                }
                KeyCode::F3 => self
                    .emulation
                    .emulator
                    .set_speed(EmulationMode::FrameCount(1)),
                KeyCode::F4 => self
                    .emulation
                    .emulator
                    .set_speed(EmulationMode::FrameCount(2)),
                KeyCode::F5 => self.emulation.emulator.set_speed(EmulationMode::Max),
                KeyCode::F6 => {
                    self.enable_frame_trace = !self.enable_frame_trace;
                    self.update_window_title();
                }
                KeyCode::F9 => {
                    self.enable_joy_keyboard_layer = !self.enable_joy_keyboard_layer;
                    self.update_window_title();
                }
                KeyCode::Insert => self.emulation.emulator.play_tape(),
                KeyCode::Delete => self.emulation.emulator.stop_tape(),
                KeyCode::Escape => self.unlock_mouse(),
                _ => {}
            }
        }

        // Highest priority to lowest, matching legacy SDL backend behavior
        if let Some(key) = map_kempston_key(
            code,
            self.kempston_enabled && self.enable_joy_keyboard_layer,
        ) {
            self.emulation.emulator.send_kempston_key(key, pressed);
            return;
        }
        if let Some((num, key)) = map_sinclair_key(code, self.enable_joy_keyboard_layer) {
            self.emulation.emulator.send_sinclair_key(num, key, pressed);
            return;
        }
        if let Some(key) = map_zx_key(code) {
            self.emulation.emulator.send_key(key, pressed);
            return;
        }
        if let Some(key) = map_compound_key(code) {
            self.emulation.emulator.send_compound_key(key, pressed);
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.graphics.is_some() {
            return;
        }

        let scale = self.emulation.settings.scale.max(1) as u32;
        let window_attributes = Window::default_attributes()
            .with_title(self.window_title())
            .with_inner_size(winit::dpi::PhysicalSize::new(
                SCREEN_WIDTH as u32 * scale,
                SCREEN_HEIGHT as u32 * scale,
            ));

        let window = match event_loop.create_window(window_attributes) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                log::error!("Failed to create window: {}", e);
                event_loop.exit();
                return;
            }
        };

        let params = wgpu_video::ScreenParams {};
        let screen = match pollster::block_on(wgpu_video::Screen::init(params, window.clone())) {
            Ok(screen) => screen,
            Err(e) => {
                log::error!("Failed to initialize renderer: {}", e);
                event_loop.exit();
                return;
            }
        };

        self.graphics = Some(Graphics { window, screen });

        // Emulate first frame right away and schedule the frame timer
        self.last_emulated_frame_time = Instant::now();
        self.emulate_and_reschedule(event_loop);
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if matches!(cause, StartCause::ResumeTimeReached { .. }) {
            self.emulate_and_reschedule(event_loop);
            if let Some(graphics) = self.graphics.as_ref() {
                graphics.window.request_redraw();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(graphics) = self.graphics.as_mut() {
                    graphics
                        .screen
                        .resize(size.width.max(1), size.height.max(1));
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(graphics) = self.graphics.as_mut() {
                    graphics
                        .screen
                        .update_canvas(self.emulation.canvas_buffer());
                    graphics
                        .screen
                        .update_screen(self.emulation.border_buffer());
                    if let Err(e) = graphics.screen.render() {
                        log::error!("Render failed: {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    let pressed = event.state == ElementState::Pressed;
                    self.handle_key_action(code, pressed, event.repeat);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed {
                    self.lock_mouse();
                }
                if self.mouse_locked {
                    if let Some(kempston_button) = map_mouse_button(button) {
                        self.emulation
                            .emulator
                            .send_mouse_button(kempston_button, state == ElementState::Pressed);
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } if self.mouse_locked => {
                let y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                };
                let direction = if y > 0.0 {
                    KempstonMouseWheelDirection::Up
                } else {
                    KempstonMouseWheelDirection::Down
                };
                self.emulation.emulator.send_mouse_wheel(direction);
            }
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::DroppedFile(path) => {
                if let Err(e) = self.emulation.load_file_autodetect(&path) {
                    log::error!("Failed to load dropped file: {:#}", e);
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion {
            delta: (xrel, yrel),
        } = event
        {
            if !self.mouse_locked {
                return;
            }

            let xrel = xrel as i32;
            let yrel = yrel as i32;

            // Change of direction requires counter reset to eliminate lag
            if self.mouse_x_counter.signum() != xrel.signum() {
                self.mouse_x_counter = xrel;
            } else {
                self.mouse_x_counter += xrel;
            }
            if self.mouse_y_counter.signum() != yrel.signum() {
                self.mouse_y_counter = yrel;
            } else {
                self.mouse_y_counter += yrel;
            }

            // Depending on sensitivity, different distance is required to move kempston mouse
            let ticks_to_move = sensitivity_to_mouse_counter_ticks(self.mouse_sensitivity) as i32;
            let xshift = self.mouse_x_counter / ticks_to_move;
            let yshift = self.mouse_y_counter / ticks_to_move;
            let xrem = self.mouse_x_counter % ticks_to_move;
            let yrem = self.mouse_y_counter % ticks_to_move;
            if xshift != 0 {
                self.mouse_x_counter = xrem;
            }
            if yshift != 0 {
                self.mouse_y_counter = yrem;
            }

            if xshift != 0 || yshift != 0 {
                let x = xshift.clamp(i8::MIN as i32, i8::MAX as i32) as i8;
                let y = yshift.clamp(i8::MIN as i32, i8::MAX as i32) as i8;
                self.emulation.emulator.send_mouse_pos_diff(x, y);
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(graphics) = self.graphics.as_ref() {
            graphics.window.request_redraw();
        }
    }
}

fn main() -> anyhow::Result<()> {
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|level_str| level_str.parse().ok())
        .unwrap_or(log::Level::Warn);

    simple_logger::init_with_level(log_level).expect("Failed to initialize logger");

    let settings = Settings::from_args();
    let emulation_state = EmulationState::from_config(settings)?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(emulation_state);
    event_loop.run_app(&mut app)?;

    Ok(())
}

fn map_zx_key(key: KeyCode) -> Option<ZXKey> {
    match key {
        KeyCode::KeyA => Some(ZXKey::A),
        KeyCode::KeyB => Some(ZXKey::B),
        KeyCode::KeyC => Some(ZXKey::C),
        KeyCode::KeyD => Some(ZXKey::D),
        KeyCode::KeyE => Some(ZXKey::E),
        KeyCode::KeyF => Some(ZXKey::F),
        KeyCode::KeyG => Some(ZXKey::G),
        KeyCode::KeyH => Some(ZXKey::H),
        KeyCode::KeyI => Some(ZXKey::I),
        KeyCode::KeyJ => Some(ZXKey::J),
        KeyCode::KeyK => Some(ZXKey::K),
        KeyCode::KeyL => Some(ZXKey::L),
        KeyCode::KeyM => Some(ZXKey::M),
        KeyCode::KeyN => Some(ZXKey::N),
        KeyCode::KeyO => Some(ZXKey::O),
        KeyCode::KeyP => Some(ZXKey::P),
        KeyCode::KeyQ => Some(ZXKey::Q),
        KeyCode::KeyR => Some(ZXKey::R),
        KeyCode::KeyS => Some(ZXKey::S),
        KeyCode::KeyT => Some(ZXKey::T),
        KeyCode::KeyU => Some(ZXKey::U),
        KeyCode::KeyV => Some(ZXKey::V),
        KeyCode::KeyW => Some(ZXKey::W),
        KeyCode::KeyX => Some(ZXKey::X),
        KeyCode::KeyY => Some(ZXKey::Y),
        KeyCode::KeyZ => Some(ZXKey::Z),
        KeyCode::Digit1 => Some(ZXKey::N1),
        KeyCode::Digit2 => Some(ZXKey::N2),
        KeyCode::Digit3 => Some(ZXKey::N3),
        KeyCode::Digit4 => Some(ZXKey::N4),
        KeyCode::Digit5 => Some(ZXKey::N5),
        KeyCode::Digit6 => Some(ZXKey::N6),
        KeyCode::Digit7 => Some(ZXKey::N7),
        KeyCode::Digit8 => Some(ZXKey::N8),
        KeyCode::Digit9 => Some(ZXKey::N9),
        KeyCode::Digit0 => Some(ZXKey::N0),
        KeyCode::Enter => Some(ZXKey::Enter),
        KeyCode::Space => Some(ZXKey::Space),
        KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(ZXKey::Shift),
        KeyCode::ControlLeft | KeyCode::ControlRight => Some(ZXKey::SymShift),
        _ => None,
    }
}

fn map_compound_key(key: KeyCode) -> Option<CompoundKey> {
    match key {
        KeyCode::ArrowUp => Some(CompoundKey::ArrowUp),
        KeyCode::ArrowDown => Some(CompoundKey::ArrowDown),
        KeyCode::ArrowLeft => Some(CompoundKey::ArrowLeft),
        KeyCode::ArrowRight => Some(CompoundKey::ArrowDown),
        KeyCode::CapsLock => Some(CompoundKey::CapsLock),
        KeyCode::Backspace => Some(CompoundKey::Delete),
        KeyCode::End => Some(CompoundKey::Break),
        _ => None,
    }
}

fn map_kempston_key(key: KeyCode, enabled: bool) -> Option<KempstonKey> {
    if !enabled {
        return None;
    }

    match key {
        KeyCode::AltLeft | KeyCode::AltRight => Some(KempstonKey::Fire),
        KeyCode::ArrowUp => Some(KempstonKey::Up),
        KeyCode::ArrowDown => Some(KempstonKey::Down),
        KeyCode::ArrowLeft => Some(KempstonKey::Left),
        KeyCode::ArrowRight => Some(KempstonKey::Right),
        _ => None,
    }
}

fn map_sinclair_key(key: KeyCode, enabled: bool) -> Option<(SinclairJoyNum, SinclairKey)> {
    if !enabled {
        return None;
    }

    match key {
        // Joy 1
        KeyCode::KeyA => Some((SinclairJoyNum::Fist, SinclairKey::Left)),
        KeyCode::KeyW => Some((SinclairJoyNum::Fist, SinclairKey::Up)),
        KeyCode::KeyS => Some((SinclairJoyNum::Fist, SinclairKey::Down)),
        KeyCode::KeyD => Some((SinclairJoyNum::Fist, SinclairKey::Right)),
        KeyCode::CapsLock => Some((SinclairJoyNum::Fist, SinclairKey::Fire)),
        // Joy 2
        KeyCode::KeyJ => Some((SinclairJoyNum::Second, SinclairKey::Left)),
        KeyCode::KeyI => Some((SinclairJoyNum::Second, SinclairKey::Up)),
        KeyCode::KeyK => Some((SinclairJoyNum::Second, SinclairKey::Down)),
        KeyCode::KeyL => Some((SinclairJoyNum::Second, SinclairKey::Right)),
        KeyCode::Enter => Some((SinclairJoyNum::Second, SinclairKey::Fire)),
        _ => None,
    }
}

fn map_mouse_button(button: MouseButton) -> Option<KempstonMouseButton> {
    match button {
        MouseButton::Left => Some(KempstonMouseButton::Left),
        MouseButton::Right => Some(KempstonMouseButton::Right),
        MouseButton::Middle => Some(KempstonMouseButton::Middle),
        MouseButton::Back | MouseButton::Forward => Some(KempstonMouseButton::Additional),
        _ => None,
    }
}

fn sensitivity_to_mouse_counter_ticks(sensitivity: usize) -> usize {
    const MIN_MOUSE_SENSITIVITY: usize = 1;
    const MAX_MOUSE_SENSITIVITY: usize = 100;

    MAX_MOUSE_SENSITIVITY / sensitivity.clamp(MIN_MOUSE_SENSITIVITY, MAX_MOUSE_SENSITIVITY)
}
