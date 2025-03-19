use crate::app::event::GlobEvent;
use crate::app::state_runner::{RunnableState, StateRunnerContext, StateRunnerError};
use device_traits::Logger;

#[derive(Default)]
pub struct InitializingStateRunner {}
impl RunnableState for InitializingStateRunner {
    fn entry<LOG: Logger>(
        &mut self,
        _ctx: &mut StateRunnerContext<LOG>
    ) {
    }
    fn update<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) -> Result<(), StateRunnerError> {

        ctx.event_queue
            .enqueue(GlobEvent::InitFinished)
            .map_err(StateRunnerError::QueueFull)?;
        //let inputs = iomanager.read_motor_inputs().map_err(|e|StateRunnerError::HomingIOError(e))?;
        //logger.debug(&str_to_display!("0: ({}) 1: ({}) 2: ({})", inputs[0].1.limit_reached, inputs[1].1.limit_reached, inputs[2].1.limit_reached));
        Ok(())
    }
    fn exit<LOG: Logger>(
        &mut self,
        _ctx: &mut StateRunnerContext<LOG>
    ) {}
}
