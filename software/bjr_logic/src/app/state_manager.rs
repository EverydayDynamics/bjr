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
            current_state: State::Default,
        };
        sr
    }
    pub fn update(&mut self, state: State, call_time: Microseconds<u64>, event_queue: EventQueue) -> Result<(),StateRunnerError> {
        if self.current_state != state {
            self.runners.get_runner(self.current_state).exit(call_time);
            self.runners.get_runner(state).entry(call_time);
            self.current_state = state;
        }
        let runner = self.runners.get_runner(self.current_state);
        runner.update(&mut self.io_manager, call_time, event_queue)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;
    use crate::app::io_manager::{Inputs, Outputs, IOManagerError};
    use crate::app::state_runner::RunnableState;
    use crate::app::control_primitives::KinState;
    use crate::app::motor_handler::{ControlMode, MotorStatus};
    use crate::app::event_queue::EventQueue;

    mock! {
        pub TestStateRunnerSelector {}
        impl<'a> StateRunnerSelector for TestStateRunnerSelector {
            fn get_runner(&mut self, state: State) -> &mut dyn RunnableState;
        }
    }
    mock! {
        pub TestRunnableState {}
        impl<'a> RunnableState for TestRunnableState {
            fn entry(&mut self, call_time: Microseconds<u64>);
            fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, event_queue: EventQueue) -> Result<(),StateRunnerError>;
            fn exit(&mut self, call_time: Microseconds<u64>);
        }
    }
    mock! {
        pub TestRunnableState2 {}
        impl<'a> RunnableState for TestRunnableState2 {
            fn entry(&mut self, call_time: Microseconds<u64>);
            fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, event_queue: EventQueue) -> Result<(),StateRunnerError>;
            fn exit(&mut self, call_time: Microseconds<u64>);
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

    #[test]
    fn test_state_runner_no_state_change() {
        let mock_iomanager = MockTestIOManager::new();
        let mut mock_state_runner_selector = MockTestStateRunnerSelector::new();
        let mut mock_testrunnablestate_default = MockTestRunnableState::new();
        let mut mock_testrunnablestate_init = MockTestRunnableState::new();
        mock_testrunnablestate_default.expect_exit()
            .with( eq(Microseconds::new(0)))
            .returning(|_|());
        mock_testrunnablestate_init.expect_entry()
            .with( eq(Microseconds::new(0)))
            .returning(|_|());
        mock_testrunnablestate_init.expect_update()
            .with(always(), eq(Microseconds::new(0)),always())
            .returning(|_,_,_|Ok(()));
        mock_state_runner_selector.expect_get_runner()
            .with(eq(State::Default))
            .return_var(Box::new(mock_testrunnablestate_default));
        mock_state_runner_selector.expect_get_runner()
            .with(eq(State::Initializing))
            .return_var(Box::new(mock_testrunnablestate_init));
        let mut test_state_runner = StateManager::new(mock_state_runner_selector, mock_iomanager);
        let result = test_state_runner.update(State::Initializing, Microseconds::new(0), get_event_queue());
        assert!(result == Ok(()));
    }
}
