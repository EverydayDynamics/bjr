use crate::app::state_runners::default_state_runner::DefaultStateRunner;
use crate::app::state_runner::RunnableState;
use crate::app::event_handler::State;
use crate::app::state_runners::homing_state_runner::HomingStateRunner;
use crate::app::state_runners::initializing_state_runner::InitializingStateRunner;

pub trait StateRunnerSelector {
    fn get_runner(&mut self, state: State) -> &mut dyn RunnableState;
}
pub struct DefaultStateRunnerSelector {
    default_state_runner: DefaultStateRunner,
    initializing_state_runner: InitializingStateRunner,
    homing_state_runner: HomingStateRunner,
}
impl DefaultStateRunnerSelector {
    pub fn new() -> Self {
        DefaultStateRunnerSelector {
            default_state_runner: Default::default(),
            initializing_state_runner: Default::default(),
            homing_state_runner: HomingStateRunner::new(),
        }
    }
}
impl StateRunnerSelector for DefaultStateRunnerSelector {
    fn get_runner(&mut self, state: State) -> &mut dyn RunnableState {
        match state {
            State::Initializing => {&mut self.initializing_state_runner}
            State::Homing => {&mut self.homing_state_runner}
            State::RunningCenterHold => {&mut self.default_state_runner}
            State::RunningCircling => {&mut self.default_state_runner}
            State::RunningTriangle => {&mut self.default_state_runner}
            State::RunningNoBall => {&mut self.default_state_runner}
            State::Deinit => {&mut self.default_state_runner}
            State::Off => {&mut self.default_state_runner}
            _ => {&mut self.default_state_runner}
        }
    }
}

