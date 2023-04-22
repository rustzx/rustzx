use crate::{
    app::events::{EventDevice, Event},
};

#[derive(Default)]
pub struct Device {}

impl EventDevice for Device {
    fn pop_event(&mut self) -> Option<Event> {
        None
    }
}
