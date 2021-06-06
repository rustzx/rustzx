//! Real events SDL backend
use super::{Event, EventDevice};
use crate::{app::settings::Settings, backends::SDL_CONTEXT};
use rustzx_core::{
    zx::{
        joy::{
            kempston::KempstonKey,
            sinclair::{SinclairJoyNum, SinclairKey},
        },
        keys::{CompoundKey, ZXKey},
        mouse::kempston::{KempstonMouseButton, KempstonMouseWheelDirection},
    },
    EmulationMode,
};
use sdl2::{
    event::Event as SdlEvent,
    keyboard::Scancode,
    mouse::{MouseButton, MouseUtil},
    EventPump,
};

/// Represents SDL Envets bakend
pub struct EventsSdl {
    event_pump: EventPump,
    mouse: MouseUtil,
    kempston_enabled: bool,
    mouse_enabled: bool,
    mouse_locked: bool,
    screen_scale: usize,
    mouse_sensitivity: usize,
    enable_joy_keyaboard_layer: bool,
    mouse_x_counter: i32,
    mouse_y_counter: i32,
}

impl EventsSdl {
    /// constructs new event backend from setttigs/
    /// Settings will be used in future for key bindings sittings
    pub fn new(settings: &Settings) -> EventsSdl {
        // init event system
        let (event_pump, mouse) = SDL_CONTEXT.with(|sdl| {
            let context = sdl.borrow_mut();
            let pump = context
                .event_pump()
                .expect("[ERROR] Sdl event pump init error");

            let mouse = context.mouse();

            (pump, mouse)
        });

        EventsSdl {
            event_pump,
            mouse,
            mouse_enabled: settings.enable_mouse,
            mouse_locked: false,
            kempston_enabled: !settings.disable_kempston,
            screen_scale: settings.scale,
            enable_joy_keyaboard_layer: false,
            mouse_sensitivity: settings.mouse_sensitivity,
            mouse_x_counter: 0,
            mouse_y_counter: 0,
        }
    }

    fn lock_mouse(&mut self) {
        if self.mouse_enabled {
            self.mouse.set_relative_mouse_mode(true);
            self.mouse.show_cursor(false);
            self.mouse_locked = true;
        }
    }

    fn unlock_mouse(&mut self) {
        if self.mouse_enabled {
            self.mouse.set_relative_mouse_mode(false);
            self.mouse.show_cursor(true);
            self.mouse_locked = false;
        }
    }

    /// returns ZX Spectum key form scancode of None if not found
    fn scancode_to_zxkey_event(&self, scancode: Option<Scancode>, pressed: bool) -> Option<Event> {
        let zxkey_event = match scancode? {
            // FEFE
            Scancode::LShift | Scancode::RShift => Some(ZXKey::Shift),
            Scancode::Z => Some(ZXKey::Z),
            Scancode::X => Some(ZXKey::X),
            Scancode::C => Some(ZXKey::C),
            Scancode::V => Some(ZXKey::V),
            // FDDE
            Scancode::A => Some(ZXKey::A),
            Scancode::S => Some(ZXKey::S),
            Scancode::D => Some(ZXKey::D),
            Scancode::F => Some(ZXKey::F),
            Scancode::G => Some(ZXKey::G),
            // FBFE
            Scancode::Q => Some(ZXKey::Q),
            Scancode::W => Some(ZXKey::W),
            Scancode::E => Some(ZXKey::E),
            Scancode::R => Some(ZXKey::R),
            Scancode::T => Some(ZXKey::T),
            // F7FE
            Scancode::Num1 => Some(ZXKey::N1),
            Scancode::Num2 => Some(ZXKey::N2),
            Scancode::Num3 => Some(ZXKey::N3),
            Scancode::Num4 => Some(ZXKey::N4),
            Scancode::Num5 => Some(ZXKey::N5),
            // EFFE
            Scancode::Num0 => Some(ZXKey::N0),
            Scancode::Num9 => Some(ZXKey::N9),
            Scancode::Num8 => Some(ZXKey::N8),
            Scancode::Num7 => Some(ZXKey::N7),
            Scancode::Num6 => Some(ZXKey::N6),
            // DFFE
            Scancode::P => Some(ZXKey::P),
            Scancode::O => Some(ZXKey::O),
            Scancode::I => Some(ZXKey::I),
            Scancode::U => Some(ZXKey::U),
            Scancode::Y => Some(ZXKey::Y),
            // BFFE
            Scancode::Return => Some(ZXKey::Enter),
            Scancode::L => Some(ZXKey::L),
            Scancode::K => Some(ZXKey::K),
            Scancode::J => Some(ZXKey::J),
            Scancode::H => Some(ZXKey::H),
            // 7FFE
            Scancode::Space => Some(ZXKey::Space),
            Scancode::LCtrl | Scancode::RCtrl => Some(ZXKey::SymShift),
            Scancode::M => Some(ZXKey::M),
            Scancode::N => Some(ZXKey::N),
            Scancode::B => Some(ZXKey::B),
            _ => None,
        };

        zxkey_event.map(|k| Event::ZXKey(k, pressed))
    }

    fn scancode_to_compound_key_event(
        &self,
        scancode: Option<Scancode>,
        pressed: bool,
    ) -> Option<Event> {
        let compound_key_event = match scancode? {
            Scancode::Up => Some(CompoundKey::ArrowUp),
            Scancode::Down => Some(CompoundKey::ArrowDown),
            Scancode::Left => Some(CompoundKey::ArrowLeft),
            Scancode::Right => Some(CompoundKey::ArrowDown),
            Scancode::CapsLock => Some(CompoundKey::CapsLock),
            Scancode::Backspace => Some(CompoundKey::Delete),
            Scancode::End => Some(CompoundKey::Break),
            _ => None,
        };

        compound_key_event.map(|k| Event::CompoundKey(k, pressed))
    }

    /// returns kempston key form scancode of None if not found
    fn scancode_to_kempston_event(
        &self,
        scancode: Option<Scancode>,
        pressed: bool,
    ) -> Option<Event> {
        if !(self.kempston_enabled && self.enable_joy_keyaboard_layer) {
            return None;
        }

        let kempston_event = match scancode? {
            Scancode::LAlt | Scancode::RAlt => Some(KempstonKey::Fire),
            Scancode::Up => Some(KempstonKey::Up),
            Scancode::Down => Some(KempstonKey::Down),
            Scancode::Left => Some(KempstonKey::Left),
            Scancode::Right => Some(KempstonKey::Right),
            _ => None,
        };

        kempston_event.map(|k| Event::Kempston(k, pressed))
    }

    fn scancode_to_sinclair_event(
        &self,
        scancode: Option<Scancode>,
        pressed: bool,
    ) -> Option<Event> {
        if !self.enable_joy_keyaboard_layer {
            return None;
        }

        let sinclair_event = match scancode? {
            // Joy 1
            Scancode::A => Some((SinclairJoyNum::Fist, SinclairKey::Left)),
            Scancode::W => Some((SinclairJoyNum::Fist, SinclairKey::Up)),
            Scancode::S => Some((SinclairJoyNum::Fist, SinclairKey::Down)),
            Scancode::D => Some((SinclairJoyNum::Fist, SinclairKey::Right)),
            Scancode::CapsLock => Some((SinclairJoyNum::Fist, SinclairKey::Fire)),
            // Joy 2
            Scancode::J => Some((SinclairJoyNum::Second, SinclairKey::Left)),
            Scancode::I => Some((SinclairJoyNum::Second, SinclairKey::Up)),
            Scancode::K => Some((SinclairJoyNum::Second, SinclairKey::Down)),
            Scancode::L => Some((SinclairJoyNum::Second, SinclairKey::Right)),
            Scancode::Return => Some((SinclairJoyNum::Second, SinclairKey::Fire)),
            _ => None,
        };

        sinclair_event.map(|(n, k)| Event::Sinclair(n, k, pressed))
    }

    fn scancode_to_emulator_event(
        &mut self,
        scancode: Option<Scancode>,
        pressed: bool,
    ) -> Option<Event> {
        if let (Some(code), true) = (scancode, pressed) {
            match code {
                Scancode::F1 => Some(Event::QuickSave),
                Scancode::F2 => Some(Event::QuickLoad),
                Scancode::F3 => Some(Event::ChangeSpeed(EmulationMode::FrameCount(1))),
                Scancode::F4 => Some(Event::ChangeSpeed(EmulationMode::FrameCount(2))),
                Scancode::F5 => Some(Event::ChangeSpeed(EmulationMode::Max)),
                Scancode::F6 => Some(Event::SwitchFrameTrace),
                Scancode::F9 => {
                    self.enable_joy_keyaboard_layer = !self.enable_joy_keyaboard_layer;
                    Some(Event::ChangeJoyKeyboardLayer(
                        self.enable_joy_keyaboard_layer,
                    ))
                }
                Scancode::Insert => Some(Event::InsertTape),
                Scancode::Delete => Some(Event::StopTape),
                Scancode::Escape => {
                    self.unlock_mouse();
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

impl EventDevice for EventsSdl {
    /// get last event
    fn pop_event(&mut self) -> Option<Event> {
        if let Some(event) = self.event_pump.poll_event() {
            // if event found
            match event {
                // exot requested
                SdlEvent::Quit { .. } => Some(Event::Exit),
                // if any key pressed
                action @ SdlEvent::KeyDown { .. } | action @ SdlEvent::KeyUp { .. } => {
                    // assemble tuple from scancode and its state
                    let (scancode, pressed) = match action {
                        SdlEvent::KeyDown { scancode: code, .. } => (code, true),
                        SdlEvent::KeyUp { scancode: code, .. } => (code, false),
                        _ => unreachable!(),
                    };

                    // Form highest priority event to lowest
                    self.scancode_to_emulator_event(scancode, pressed)
                        .or_else(|| self.scancode_to_kempston_event(scancode, pressed))
                        .or_else(|| self.scancode_to_sinclair_event(scancode, pressed))
                        .or_else(|| self.scancode_to_zxkey_event(scancode, pressed))
                        .or_else(|| self.scancode_to_compound_key_event(scancode, pressed))
                }
                SdlEvent::MouseMotion { xrel, yrel, .. } => {
                    // Change of direction  requires counter reset to elimiate lag
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

                    // Depending on sensitivity, diffent distance is required to move
                    // kempston mouse
                    let ticks_to_move =
                        sensitivity_to_mouse_counter_ticks(self.mouse_sensitivity) as i32;
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

                    if self.mouse_locked {
                        let x = xshift.clamp(i8::MIN as i32, i8::MAX as i32) as i8;
                        let y = yshift.clamp(i8::MIN as i32, i8::MAX as i32) as i8;
                        Some(Event::MouseMove { x, y })
                    } else {
                        None
                    }
                }
                SdlEvent::MouseButtonDown { mouse_btn, .. } => {
                    self.lock_mouse();
                    if self.mouse_locked {
                        sdl_mouse_button_to_kempston(mouse_btn)
                            .map(|button| Event::MouseButton(button, true))
                    } else {
                        None
                    }
                }
                SdlEvent::MouseButtonUp { mouse_btn, .. } => {
                    if self.mouse_locked {
                        sdl_mouse_button_to_kempston(mouse_btn)
                            .map(|button| Event::MouseButton(button, false))
                    } else {
                        None
                    }
                }
                SdlEvent::MouseWheel { y, .. } => {
                    if self.mouse_locked {
                        if y > 0 {
                            Some(Event::MouseWheel(KempstonMouseWheelDirection::Up))
                        } else {
                            Some(Event::MouseWheel(KempstonMouseWheelDirection::Down))
                        }
                    } else {
                        None
                    }
                }
                SdlEvent::DropFile { filename, .. } => Some(Event::OpenFile(filename.into())),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn sdl_mouse_button_to_kempston(button: MouseButton) -> Option<KempstonMouseButton> {
    match button {
        MouseButton::Left => Some(KempstonMouseButton::Left),
        MouseButton::Right => Some(KempstonMouseButton::Right),
        MouseButton::Middle => Some(KempstonMouseButton::Middle),
        MouseButton::X1 => Some(KempstonMouseButton::Additional),
        _ => None,
    }
}

fn sensitivity_to_mouse_counter_ticks(sensitivity: usize) -> usize {
    const MIN_MOUSE_SENSITIVITY: usize = 1;
    const MAX_MOUSE_SENSITIVITY: usize = 100;

    MAX_MOUSE_SENSITIVITY / sensitivity.clamp(MIN_MOUSE_SENSITIVITY, MAX_MOUSE_SENSITIVITY)
}
