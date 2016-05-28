use std::collections::VecDeque;
use utils::{EmulationSpeed, Clocks};

pub enum EventKind {
    Accelerate(EmulationSpeed),
    Deaccelerate,
}

pub struct Event {
    pub kind: EventKind,
    pub time: Clocks,
}

impl Event {
    pub fn new(kind: EventKind, time: Clocks) -> Event {
        Event {
            kind: kind,
            time: time,
        }
    }
}

pub struct EventQueue {
    deque: VecDeque<Event>,
}

impl EventQueue {
    pub fn new() -> EventQueue {
        EventQueue {
            deque: VecDeque::new(),
        }
    }
    pub fn send_event(&mut self, e: Event) {
        self.deque.push_back(e);
    }

    pub fn receive_event(&mut self) -> Option<Event> {
        self.deque.pop_front()
    }
}
