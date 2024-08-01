use bsp_traits::MotorEnabler;
use embedded_time::duration::Microseconds;
use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::IOManager;
use crate::app::state_runner::{RunnableState, StateRunnerError};

#[derive(Default)]
pub struct InitializingStateRunner {
}
impl RunnableState for InitializingStateRunner {
    fn entry(&mut self, _call_time: Microseconds<u64>, _motor_enabler: &mut dyn MotorEnabler) {}
    fn update(&mut self, _iomanager: &mut dyn IOManager, _call_time: Microseconds<u64>, event_queue: EventQueue) -> Result<(),StateRunnerError> {
        event_queue.enqueue(GlobEvent::InitFinished).map_err(|event|StateRunnerError::QueueFull(event))?;
        Ok(())
    }
    fn exit(&mut self, _call_time: Microseconds<u64>) {}
}
