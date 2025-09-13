use crate::app::state_runner::{RunnableState, StateRunnerContext, StateRunnerError};
use device_traits::Logger;

#[derive(Default)]
pub struct DefaultStateRunner {}
impl RunnableState for DefaultStateRunner {
    fn entry<LOG: Logger>(&mut self, _ctx: &mut StateRunnerContext<LOG>) {}
    fn update<LOG: Logger>(
        &mut self,
        _ctx: &mut StateRunnerContext<LOG>,
    ) -> Result<(), StateRunnerError> {
        Ok(())
    }
    fn exit<LOG: Logger>(&mut self, _ctx: &mut StateRunnerContext<LOG>) {}
}
