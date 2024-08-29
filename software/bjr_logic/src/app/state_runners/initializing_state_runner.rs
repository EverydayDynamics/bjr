use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerContext, StateRunnerError};
use bsp_traits::Logger;
use bsp_traits::MotorEnabler;
use embedded_time::duration::Microseconds;
use crate::app::telemetry_handler::TelemetryBuilder;

#[derive(Default)]
pub struct InitializingStateRunner {}
impl RunnableState for InitializingStateRunner {
    fn entry<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
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
        ctx: &mut StateRunnerContext<LOG>
    ) {}
}
