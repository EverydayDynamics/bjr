use crate::app::control::ControlExecutor;
use crate::app::control::pid_controller::PIDController;
use crate::app::control::setpoint_generator::SetPointGenCH;
use crate::app::control::feedforward_generator::FFGenCH;
use crate::app::state_runners::default_state_runner::DefaultStateRunner;
use crate::app::state_runner::RunnableState;
use crate::app::event_handler::State;
use crate::app::state_runners::control_state_runner::ControlStateRunner;
use crate::app::state_runners::homing_state_runner::HomingStateRunner;
use crate::app::state_runners::initializing_state_runner::InitializingStateRunner;
use crate::app::state_runners::no_ball_state_runner::NoBallStateRunner;

pub trait StateRunnerSelector {
    fn get_runner(&mut self, state: State) -> &mut dyn RunnableState;
}
pub struct DefaultStateRunnerSelector {
    default_state_runner: DefaultStateRunner,
    initializing_state_runner: InitializingStateRunner,
    homing_state_runner: HomingStateRunner,
    noball_state_runner: NoBallStateRunner,
    center_hold_control_runner: ControlStateRunner<PIDController, FFGenCH, SetPointGenCH>,
}
impl DefaultStateRunnerSelector {
    pub fn new() -> Self {
        DefaultStateRunnerSelector {
            default_state_runner: Default::default(),
            initializing_state_runner: Default::default(),
            homing_state_runner: HomingStateRunner::new(),
            noball_state_runner: NoBallStateRunner::default(),
            center_hold_control_runner: ControlStateRunner::new(ControlExecutor::new(PIDController{}, FFGenCH{}, SetPointGenCH{}))
        }
    }
}
impl StateRunnerSelector for DefaultStateRunnerSelector {
    fn get_runner(&mut self, state: State) -> &mut dyn RunnableState {
        match state {
            State::Initializing => {&mut self.initializing_state_runner}
            State::Homing => {&mut self.homing_state_runner}
            State::RunningCenterHold => {&mut self.center_hold_control_runner}
            State::RunningCircling => {&mut self.default_state_runner}
            State::RunningTriangle => {&mut self.default_state_runner}
            State::RunningNoBall => {&mut self.noball_state_runner}
            State::Deinit => {&mut self.default_state_runner}
            State::Off => {&mut self.default_state_runner}
            _ => {&mut self.default_state_runner}
        }
    }
}

