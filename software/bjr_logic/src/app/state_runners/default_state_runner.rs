use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerCommand, StateRunnerContext, StateRunnerError};
use bsp_traits::{Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use crate::app::telemetry_handler::TelemetryBuilder;

#[derive(Default)]
pub struct DefaultStateRunner {}
impl RunnableState for DefaultStateRunner {
    fn entry<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) {
    }
    fn update<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) -> Result<(), StateRunnerError> {
        Ok(())
    }
    fn exit<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>
    ) {}
}
