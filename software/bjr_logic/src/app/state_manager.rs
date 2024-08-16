use bsp_traits::{Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use crate::app::event_handler::State;
use crate::app::event_queue::{EventQueue, get_event_queue};
use crate::app::io_manager::IOManager;
use crate::app::state_runner_selector::StateRunnerSelector;
use crate::app::state_runner::StateRunnerError;

pub struct StateManager<STRS, IOMAN> {
    runners: STRS,
    io_manager: IOMAN,
    current_state: State,
}
impl<STRS, IOMAN> StateManager<STRS, IOMAN>
where
    STRS: StateRunnerSelector,
    IOMAN: IOManager,
{
    pub fn new(runners: STRS, io_manager: IOMAN) -> Self{
        let sr = StateManager {
            runners,
            io_manager,
            current_state: State::Initializing,
        };
        sr
    }
    pub fn update(&mut self, state: State, call_time: Microseconds<u64>, event_queue: EventQueue, motor_enabler: &mut dyn MotorEnabler, logger: &mut dyn Logger) -> Result<(),StateRunnerError> {
        if self.current_state != state {

            self.runners.get_runner(self.current_state).exit(call_time, logger);
            self.runners.get_runner(state).entry(call_time, motor_enabler, logger);
            self.current_state = state;
        }
        let runner = self.runners.get_runner(self.current_state);
        runner.update(&mut self.io_manager, call_time, event_queue, logger)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::{mock, predicate};
    use crate::app::io_manager::{Inputs, Outputs, IOManagerError};
    use crate::app::state_runner::RunnableState;
    use crate::app::control_primitives::KinState;
    use crate::app::motor_handler::{ControlMode, MotorStatus};
    use crate::app::event_queue::EventQueue;

    mock! {
        pub TestMotorEnabler {}
        impl<'a> MotorEnabler for TestMotorEnabler {
            fn set_enable(&mut self, enable: bool);
        }
    }
    mock! {
        pub TestStateRunnerSelector {}
        impl<'a> StateRunnerSelector for TestStateRunnerSelector {
            fn get_runner(&mut self, state: State) -> &mut dyn RunnableState;
        }
    }
    mock! {
        pub TestRunnableState {}
        impl<'a> RunnableState for TestRunnableState {
            fn entry(&mut self, call_time: Microseconds<u64>, motor_enabler: &mut dyn MotorEnabler, logger: & dyn Logger);
            fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, event_queue: EventQueue, logger: & dyn Logger) -> Result<(),StateRunnerError>;
            fn exit(&mut self, call_time: Microseconds<u64>, logger: & dyn Logger);
        }
    }
    mock! {
        pub TestRunnableState2 {}
        impl<'a> RunnableState for TestRunnableState2 {
            fn entry(&mut self, call_time: Microseconds<u64>, motor_enabler: &mut dyn MotorEnabler, logger: & dyn Logger);
            fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, event_queue: EventQueue, logger: & dyn Logger) -> Result<(),StateRunnerError>;
            fn exit(&mut self, call_time: Microseconds<u64>, logger: & dyn Logger);
        }
    }

    mock! {
        pub TestIOManager {}
        impl<'a> IOManager for TestIOManager {
            fn read_all_inputs(&mut self, call_time: Microseconds<u64>) -> Result<Inputs, IOManagerError>;
            fn write_all_outputs(&mut self, outputs: Outputs) -> Result<(), IOManagerError>;
            fn read_motor_inputs(&mut self) -> Result<[(KinState, MotorStatus); 3], IOManagerError>;
            fn write_motor_outputs(&mut self, output: [Option<(KinState, ControlMode)>;3]) -> Result<(), IOManagerError>;
            fn reset_motor_pos(&mut self, motor_idx: usize) -> Result<(), IOManagerError>;
        }
    }

}
