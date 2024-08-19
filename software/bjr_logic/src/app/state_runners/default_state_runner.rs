use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerError};
use bsp_traits::{Logger, MotorEnabler};
use embedded_time::duration::Microseconds;

#[derive(Default)]
pub struct DefaultStateRunner {}
impl RunnableState for DefaultStateRunner {
    fn entry<LOG: Logger>(
        &mut self,
        _call_time: Microseconds<u64>,
        _motor_enabler: &mut dyn MotorEnabler,
        _logger: &mut LOG,
    ) {
    }
    fn update<LOG: Logger>(
        &mut self,
        _iomanager: &mut dyn IOManager,
        _call_time: Microseconds<u64>,
        _event_queue: EventQueue,
        _logger: &mut LOG,
        _command: &StateRunnerCommand,
    ) -> Result<(), StateRunnerError> {
        Ok(())
    }
    fn exit<LOG: Logger>(&mut self, _call_time: Microseconds<u64>, _logger: &mut LOG) {}
}
