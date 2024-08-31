use crate::app::event::GlobEvent;
use heapless::mpmc::Q8;

static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
pub type EventQueue = &'static Q8<GlobEvent>;
pub fn get_event_queue() -> EventQueue {
    &EVENT_QUEUE
}

pub fn drain_event_queue() {
    while EVENT_QUEUE.dequeue().is_some() {}
}
