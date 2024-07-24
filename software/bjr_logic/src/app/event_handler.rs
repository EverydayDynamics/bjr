
#[derive(Clone, Copy)]
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
}
impl EventHandler {
    pub fn new() -> Self {
        EventHandler {
            state: State::Off,
        }
    }
    pub fn handle_events(&mut self) -> Option<State>{
        None
    }
}