use crate::app::event_handler::State;
use crate::app::state_runner::{RunnableState, StateRunnerContext, StateRunnerError};
use crate::app::state_runner_selector::StateRunnerSelector;
use device_traits::Logger;

pub struct StateManager<STRS> {
    runners: STRS,
    current_state: State,
}
impl<STRS> StateManager<STRS>
where
    STRS: StateRunnerSelector,
{
    pub fn new(runners: STRS) -> Self {
        StateManager {
            runners,
            current_state: State::Initializing,
        }
    }
    pub fn update<LOG: Logger>(
        &mut self,
        state: State,
        ctx: &mut StateRunnerContext<LOG>,
    ) -> Result<(), StateRunnerError> {
        if self.current_state != state {
            self.runners
                .get_runner(self.current_state)
                .exit(ctx);
            self.runners
                .get_runner(state)
                .entry(ctx);
            self.current_state = state;
        }
        let runner = self.runners.get_runner(self.current_state);
        runner.update(
            ctx
        )?;
        Ok(())
    }
}
