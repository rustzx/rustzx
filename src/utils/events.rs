use std::collections::VecDeque;
use utils::{EmulationSpeed, Clocks};

/// Type of happened event
pub enum EventKind {
    Accelerate(EmulationSpeed),
    Deaccelerate,
    FastTapeLoad,
}

/// event, have information about kind and time of event
pub struct Event {
    pub kind: EventKind,
    pub time: Clocks,
}

impl Event {
    /// constructs new event
    pub fn new(kind: EventKind, time: Clocks) -> Event {
        Event {
            kind: kind,
            time: time,
        }
    }
}

/// Queue-based event container
pub struct EventQueue {
    deque: VecDeque<Event>,
}

impl EventQueue {
    /// cnstructs new EventQueue
    pub fn new() -> EventQueue {
        EventQueue { deque: VecDeque::new() }
    }
    /// addd new event
    pub fn send_event(&mut self, e: Event) {
        self.deque.push_back(e);
    }
    /// pops last event from deque
    pub fn receive_event(&mut self) -> Option<Event> {
        self.deque.pop_front()
    }
    /// returns true if container is empty
    pub fn is_empty(&self) -> bool {
        self.deque.is_empty()
    }
    /// removes all events
    pub fn clear(&mut self) {
        self.deque.clear();
    }
}
