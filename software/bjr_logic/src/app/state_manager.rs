use bsp_traits::{Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use crate::app::event_handler::State;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::state_runner_selector::StateRunnerSelector;
use crate::app::state_runner::{StateRunnerCommand, StateRunnerError};

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
        
        StateManager {
            runners,
            io_manager,
            current_state: State::Initializing,
        }
    }
    pub fn update(&mut self, state: State, call_time: Microseconds<u64>, event_queue: EventQueue, motor_enabler: &mut dyn MotorEnabler, logger: &mut dyn Logger, command: &StateRunnerCommand) -> Result<(),StateRunnerError> {
        if self.current_state != state {

            self.runners.get_runner(self.current_state).exit(call_time, logger);
            self.runners.get_runner(state).entry(call_time, motor_enabler, logger);
            self.current_state = state;
        }
        let runner = self.runners.get_runner(self.current_state);
        runner.update(&mut self.io_manager, call_time, event_queue, logger, command)?;
        Ok(())
    }
}

