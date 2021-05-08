//! Real events SDL backend
use super::{Event, EventDevice};
use crate::{app::settings::Settings, backends::SDL_CONTEXT};
use rustzx_core::{
    utils::EmulationSpeed,
    zx::{joy::kempston::KempstonKey, keys::*},
};
use sdl2::{event::Event as SdlEvent, keyboard::Scancode, EventPump};

/// Represents SDL Envets bakend
pub struct EventsSdl {
    event_pump: EventPump,
}

impl EventsSdl {
    /// constructs new event backend from setttigs/
    /// Settings will be used in future for key bindings sittings
    pub fn new(_settings: &Settings) -> EventsSdl {
        // init event system
        let mut pump = None;
        SDL_CONTEXT.with(|sdl| {
            pump = sdl.borrow_mut().event_pump().ok();
        });
        if let Some(pump) = pump {
            EventsSdl { event_pump: pump }
        } else {
            panic!("[ERROR] Sdl event pump init error");
        }
    }

    /// returns ZX Spectum key form scancode of None if not found
    fn scancode_to_zxkey(&self, scancode: Option<Scancode>) -> Option<ZXKey> {
        match scancode? {
            // FEFE
            Scancode::LShift | Scancode::RShift => Some(ZX_KEY_SHIFT),
            Scancode::Z => Some(ZX_KEY_Z),
            Scancode::X => Some(ZX_KEY_X),
            Scancode::C => Some(ZX_KEY_C),
            Scancode::V => Some(ZX_KEY_V),
            // FDDE
            Scancode::A => Some(ZX_KEY_A),
            Scancode::S => Some(ZX_KEY_S),
            Scancode::D => Some(ZX_KEY_D),
            Scancode::F => Some(ZX_KEY_F),
            Scancode::G => Some(ZX_KEY_G),
            // FBFE
            Scancode::Q => Some(ZX_KEY_Q),
            Scancode::W => Some(ZX_KEY_W),
            Scancode::E => Some(ZX_KEY_E),
            Scancode::R => Some(ZX_KEY_R),
            Scancode::T => Some(ZX_KEY_T),
            // F7FE
            Scancode::Num1 => Some(ZX_KEY_1),
            Scancode::Num2 => Some(ZX_KEY_2),
            Scancode::Num3 => Some(ZX_KEY_3),
            Scancode::Num4 => Some(ZX_KEY_4),
            Scancode::Num5 => Some(ZX_KEY_5),
            // EFFE
            Scancode::Num0 => Some(ZX_KEY_0),
            Scancode::Num9 => Some(ZX_KEY_9),
            Scancode::Num8 => Some(ZX_KEY_8),
            Scancode::Num7 => Some(ZX_KEY_7),
            Scancode::Num6 => Some(ZX_KEY_6),
            // DFFE
            Scancode::P => Some(ZX_KEY_P),
            Scancode::O => Some(ZX_KEY_O),
            Scancode::I => Some(ZX_KEY_I),
            Scancode::U => Some(ZX_KEY_U),
            Scancode::Y => Some(ZX_KEY_Y),
            // BFFE
            Scancode::Return => Some(ZX_KEY_ENTER),
            Scancode::L => Some(ZX_KEY_L),
            Scancode::K => Some(ZX_KEY_K),
            Scancode::J => Some(ZX_KEY_J),
            Scancode::H => Some(ZX_KEY_H),
            // 7FFE
            Scancode::Space => Some(ZX_KEY_SPACE),
            Scancode::LCtrl | Scancode::RCtrl => Some(ZX_KEY_SYM_SHIFT),
            Scancode::M => Some(ZX_KEY_M),
            Scancode::N => Some(ZX_KEY_N),
            Scancode::B => Some(ZX_KEY_B),
            _ => None,
        }
    }

    /// returns kempston key form scancode of None if not found
    fn scancode_to_joy(&self, scancode: Option<Scancode>) -> Option<KempstonKey> {
        match scancode? {
            Scancode::LAlt | Scancode::RAlt => Some(KempstonKey::Fire),
            Scancode::Up => Some(KempstonKey::Up),
            Scancode::Down => Some(KempstonKey::Down),
            Scancode::Left => Some(KempstonKey::Left),
            Scancode::Right => Some(KempstonKey::Right),
            _ => None,
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
                    let (scancode, state) = match action {
                        SdlEvent::KeyDown { scancode: code, .. } => (code, true),
                        SdlEvent::KeyUp { scancode: code, .. } => (code, false),
                        _ => unreachable!(),
                    };
                    if let Some(key) = self.scancode_to_zxkey(scancode) {
                        // if zx spectrum key found
                        Some(Event::GameKey(key, state))
                    } else if let Some(key) = self.scancode_to_joy(scancode) {
                        // of kempston key found
                        Some(Event::Kempston(key, state))
                    } else {
                        // if speial keys are used
                        if state {
                            if let Some(code) = scancode {
                                match code {
                                    // speed control
                                    Scancode::F3 => {
                                        Some(Event::ChangeSpeed(EmulationSpeed::Definite(1)))
                                    }
                                    Scancode::F4 => {
                                        Some(Event::ChangeSpeed(EmulationSpeed::Definite(2)))
                                    }
                                    Scancode::F5 => Some(Event::ChangeSpeed(EmulationSpeed::Max)),
                                    // debug info control
                                    Scancode::F6 => Some(Event::SwitchDebug),
                                    // tape control
                                    Scancode::Insert => Some(Event::InsertTape),
                                    Scancode::Delete => Some(Event::StopTape),
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
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
