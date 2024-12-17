use device_traits::{Logger};
use crate::app::control::feedforward_generator::FFGenCH;
use crate::app::control::pid_controller::PIDController;
use crate::app::control::setpoint_generator::{SetPointGenCH, SetPointGenCircling};
use crate::app::event_handler::State;
use crate::app::state_runner::{RunnableState, StateRunnerContext, StateRunnerError};
use crate::app::state_runners::control_state_runner::ControlStateRunner;
use crate::app::state_runners::default_state_runner::DefaultStateRunner;
use crate::app::state_runners::feedforward_state_runner::FeedforwardStateRunner;
use crate::app::state_runners::homing_state_runner::HomingStateRunner;
use crate::app::state_runners::initializing_state_runner::InitializingStateRunner;

pub trait StateRunnerSelector {
    fn get_runner(&mut self, state: State) -> &mut StateRunnerWrapper;
}
pub enum StateRunnerWrapper {
    Default(DefaultStateRunner),
    Initializing(InitializingStateRunner),
    Homing(HomingStateRunner),
    CenterHoldControl(ControlStateRunner<PIDController, FFGenCH, SetPointGenCH>),
    CirclingControl(ControlStateRunner<PIDController, FFGenCH, SetPointGenCircling>),
    Feedforward(FeedforwardStateRunner),
}
impl RunnableState for StateRunnerWrapper {
    fn entry<LOG: Logger>(&mut self, ctx: &mut StateRunnerContext<LOG>) {
        match self {
            StateRunnerWrapper::Default(runner) => {runner.entry(ctx)}
            StateRunnerWrapper::Initializing(runner) => {runner.entry(ctx)}
            StateRunnerWrapper::CenterHoldControl(runner) => {runner.entry(ctx)}
            StateRunnerWrapper::CirclingControl(runner) => {runner.entry(ctx)}
            StateRunnerWrapper::Feedforward(runner) => {runner.entry(ctx)}
            StateRunnerWrapper::Homing(runner) => {runner.entry(ctx)}
        }
    }

    fn update<LOG: Logger>(&mut self, ctx: &mut StateRunnerContext<LOG>) -> Result<(), StateRunnerError> {
        match self {
            StateRunnerWrapper::Default(runner) => {runner.update(ctx)}
            StateRunnerWrapper::Initializing(runner) => {runner.update(ctx)}
            StateRunnerWrapper::CenterHoldControl(runner) => {runner.update(ctx)}
            StateRunnerWrapper::CirclingControl(runner) => {runner.update(ctx)}
            StateRunnerWrapper::Feedforward(runner) => {runner.update(ctx)}
            StateRunnerWrapper::Homing(runner) => {runner.update(ctx)}
        }
    }

    fn exit<LOG: Logger>(&mut self, ctx: &mut StateRunnerContext<LOG>) {
        match self {
            StateRunnerWrapper::Default(runner) => {runner.exit(ctx)}
            StateRunnerWrapper::Initializing(runner) => {runner.exit(ctx)}
            StateRunnerWrapper::CenterHoldControl(runner) => {runner.exit(ctx)}
            StateRunnerWrapper::CirclingControl(runner) => {runner.exit(ctx)}
            StateRunnerWrapper::Feedforward(runner) => {runner.exit(ctx)}
            StateRunnerWrapper::Homing(runner) => {runner.exit(ctx)}
        }
    }
}
pub struct DefaultStateRunnerSelector {
    default_state_runner: StateRunnerWrapper,
    initializing_state_runner: StateRunnerWrapper,
    homing_state_runner: StateRunnerWrapper,
    center_hold_control_runner: StateRunnerWrapper,
    circling_control_runner: StateRunnerWrapper,
    feedforward_state_runner: StateRunnerWrapper,
}
impl Default for DefaultStateRunnerSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultStateRunnerSelector {
    pub fn new() -> Self {
        DefaultStateRunnerSelector {
            default_state_runner: StateRunnerWrapper::Default(Default::default()),
            initializing_state_runner: StateRunnerWrapper::Initializing(Default::default()),
            homing_state_runner: StateRunnerWrapper::Homing(HomingStateRunner::new()),
            center_hold_control_runner: StateRunnerWrapper::CenterHoldControl(ControlStateRunner::new(
                PIDController::default(),
                FFGenCH {},
                SetPointGenCH {},
            )),
            circling_control_runner: StateRunnerWrapper::CirclingControl(ControlStateRunner::new(
                PIDController::default(),
                FFGenCH {},
                SetPointGenCircling::default(),
            )),
            feedforward_state_runner: StateRunnerWrapper::Feedforward(Default::default()),
        }
    }
}
impl StateRunnerSelector for DefaultStateRunnerSelector {
    fn get_runner(&mut self, state: State) -> &mut StateRunnerWrapper {
        match state {
            State::Initializing => &mut self.initializing_state_runner,
            State::Homing => &mut self.homing_state_runner,
            State::RunningCenterHold => &mut self.center_hold_control_runner,
            State::RunningCircling => &mut self.circling_control_runner,
            State::RunningTriangle => &mut self.center_hold_control_runner,
            State::FeedForward => &mut self.feedforward_state_runner,
            State::Deinit => &mut self.default_state_runner,
            State::Off => &mut self.default_state_runner,
            _ => &mut self.default_state_runner,
        }
    }
}
