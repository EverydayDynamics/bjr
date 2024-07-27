use embedded_time::duration::Microseconds;
use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::{IOManager, IOManagerError};

#[derive(PartialEq, Debug)]
pub enum StateRunnerError {
    HomingIOError(IOManagerError),
    HomingOverrun,
    HomingLimistSWStuckAtSafePos,
    HomingUnexpectedStopGoingToSafePos,
    HomingInErrorState,

}

pub trait RunnableState {
    fn entry(&mut self, call_time: Microseconds<u64>);
    fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, event_queue: EventQueue) -> Result<(),StateRunnerError>;
    fn exit(&mut self, call_time: Microseconds<u64>);
}
