use bsp_traits::{Logger, MotorEnabler};
use embedded_time::duration::Microseconds;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerError};

#[derive(Default)]
pub struct FeedforwardStateRunner {
}
impl RunnableState for FeedforwardStateRunner {
    fn entry(&mut self, _call_time: Microseconds<u64>, _motor_enabler: &mut dyn MotorEnabler, logger: & dyn Logger) {}
    fn update(&mut self, _iomanager: &mut dyn IOManager, _call_time: Microseconds<u64>, _event_queue: EventQueue, logger: & dyn Logger) -> Result<(),StateRunnerError> {
        Ok(())
    }
    fn exit(&mut self, _call_time: Microseconds<u64>, logger: & dyn Logger) {}
}
