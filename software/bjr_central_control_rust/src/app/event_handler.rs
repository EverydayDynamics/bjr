use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;

pub enum State {
    Initializing,
    Homing,
    RunningCenterHold,
    RunningCircling,
    RunningTriangle,
    RunningNoBall,
    Deinit,
    Off,
}

pub struct EventHandler {
    state: State,
    event_queue: &'static Q8<GlobEvent>
}
impl EventHandler {
    pub fn new(event_queue: &'static Q8<GlobEvent>) -> Self {
        EventHandler {
            state: State::Off,
            event_queue
        }
    }
    pub fn handle_events(&mut self) -> Option<State>{
        return None;
    }
}