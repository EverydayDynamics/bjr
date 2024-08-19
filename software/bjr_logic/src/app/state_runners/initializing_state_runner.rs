use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerError};
use bsp_traits::Logger;
use bsp_traits::MotorEnabler;
use embedded_time::duration::Microseconds;
#[derive(Default)]
pub struct InitializingStateRunner {}
impl RunnableState for InitializingStateRunner {
    fn entry<LOG: Logger>(
        &mut self,
        _call_time: Microseconds<u64>,
        _motor_enabler: &mut dyn MotorEnabler,
        logger: &mut LOG,
    ) {
    }
    fn update<LOG: Logger>(
        &mut self,
        iomanager: &mut dyn IOManager,
        _call_time: Microseconds<u64>,
        event_queue: EventQueue,
        logger: &mut LOG,
        _command: &StateRunnerCommand,
    ) -> Result<(), StateRunnerError> {
        event_queue
            .enqueue(GlobEvent::InitFinished)
            .map_err(StateRunnerError::QueueFull)?;
        //let inputs = iomanager.read_motor_inputs().map_err(|e|StateRunnerError::HomingIOError(e))?;
        //logger.debug(&str_to_display!("0: ({}) 1: ({}) 2: ({})", inputs[0].1.limit_reached, inputs[1].1.limit_reached, inputs[2].1.limit_reached));
        Ok(())
    }
    fn exit<LOG: Logger>(&mut self, _call_time: Microseconds<u64>, logger: &mut LOG) {}
}
