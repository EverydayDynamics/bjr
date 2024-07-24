use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;

static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
pub fn get_event_queue() -> &'static Q8<GlobEvent> {
    &EVENT_QUEUE
}

pub fn drain_event_queue()  {
    while EVENT_QUEUE.dequeue().is_some(){}

}
