use embedded_time::duration::Microseconds;
use bsp_traits::{Logger, MotorEnabler};
use crate::app::control::feedforward_generator::FFGenCH;
use crate::app::control::pid_controller::PIDController;
use crate::app::control::setpoint_generator::SetPointGenCH;
use crate::app::event_handler::State;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerError};
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
    Feedforward(FeedforwardStateRunner),
}
impl RunnableState for StateRunnerWrapper {
    fn entry<LOG: Logger>(&mut self, call_time: Microseconds<u64>, motor_enabler: &mut dyn MotorEnabler, logger: &mut LOG) {
        match self {
            StateRunnerWrapper::Default(runner) => {runner.entry(call_time,motor_enabler, logger)}
            StateRunnerWrapper::Initializing(runner) => {runner.entry(call_time,motor_enabler, logger)}
            StateRunnerWrapper::CenterHoldControl(runner) => {runner.entry(call_time,motor_enabler, logger)}
            StateRunnerWrapper::Feedforward(runner) => {runner.entry(call_time,motor_enabler, logger)}
            StateRunnerWrapper::Homing(runner) => {runner.entry(call_time,motor_enabler, logger)}
        }
    }

    fn update<LOG: Logger>(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, event_queue: EventQueue, logger: &mut LOG, command: &StateRunnerCommand) -> Result<(), StateRunnerError> {
        match self {
            StateRunnerWrapper::Default(runner) => {runner.update(iomanager, call_time, event_queue, logger,command)}
            StateRunnerWrapper::Initializing(runner) => {runner.update(iomanager, call_time, event_queue, logger,command)}
            StateRunnerWrapper::CenterHoldControl(runner) => {runner.update(iomanager, call_time, event_queue, logger,command)}
            StateRunnerWrapper::Feedforward(runner) => {runner.update(iomanager, call_time, event_queue, logger,command)}
            StateRunnerWrapper::Homing(runner) => {runner.update(iomanager, call_time, event_queue, logger,command)}
        }
    }

    fn exit<LOG: Logger>(&mut self, call_time: Microseconds<u64>, logger: &mut LOG) {
        match self {
            StateRunnerWrapper::Default(runner) => {runner.exit(call_time, logger)}
            StateRunnerWrapper::Initializing(runner) => {runner.exit(call_time, logger)}
            StateRunnerWrapper::CenterHoldControl(runner) => {runner.exit(call_time, logger)}
            StateRunnerWrapper::Feedforward(runner) => {runner.exit(call_time, logger)}
            StateRunnerWrapper::Homing(runner) => {runner.exit(call_time, logger)}
        }
    }
}
pub struct DefaultStateRunnerSelector {
    default_state_runner: StateRunnerWrapper,
    initializing_state_runner: StateRunnerWrapper,
    homing_state_runner: StateRunnerWrapper,
    center_hold_control_runner: StateRunnerWrapper,
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
            State::RunningCircling => &mut self.center_hold_control_runner,
            State::RunningTriangle => &mut self.center_hold_control_runner,
            State::FeedForward => &mut self.feedforward_state_runner,
            State::Deinit => &mut self.default_state_runner,
            State::Off => &mut self.default_state_runner,
            _ => &mut self.default_state_runner,
        }
    }
}
