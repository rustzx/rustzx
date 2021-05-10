//! Real events SDL backend
use super::{Event, EventDevice};
use crate::{app::settings::Settings, backends::SDL_CONTEXT};
use rustzx_core::{
    zx::{joy::kempston::KempstonKey, keys::ZXKey},
    EmulationSpeed,
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
