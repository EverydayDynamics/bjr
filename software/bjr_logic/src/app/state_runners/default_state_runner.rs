use embedded_time::duration::Microseconds;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerError};

#[derive(Default)]
pub struct DefaultStateRunner {
}
impl RunnableState for DefaultStateRunner {
    fn entry(&mut self, call_time: Microseconds<u64>) {}
    fn update(&mut self, _iomanager: &mut dyn IOManager, _call_time: Microseconds<u64>) -> Result<(),StateRunnerError> {
        Ok(())
    }
    fn exit(&mut self, call_time: Microseconds<u64>) {}
}
