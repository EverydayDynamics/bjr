use core::fmt::{Display, Formatter};
use embedded_time::duration::Microseconds;
use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::{IOManager, IOManagerError};
use crate::app::severity_trait::{ErrorSeverity, Severity};

#[derive(PartialEq,Copy, Clone)]
pub enum StateRunnerError {
    HomingIOError(IOManagerError),
    HomingOverrun,
    HomingLimistSWStuckAtSafePos,
    HomingUnexpectedStopGoingToSafePos,
    HomingInErrorState,
    QueueFull(GlobEvent),

}
impl Display for StateRunnerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            StateRunnerError::HomingIOError(e) => {write!(f, "Homing IO error: {}", e)}
            StateRunnerError::HomingOverrun => {write!(f, "Homing Overrun")}
            StateRunnerError::HomingLimistSWStuckAtSafePos => {write!(f, "Homing Limit switch stuck at safet position")}
            StateRunnerError::HomingUnexpectedStopGoingToSafePos => {write!(f, "Homing Unexpectedly stopped while going to safe position")}
            StateRunnerError::HomingInErrorState => {write!(f, "Homing is in error state")}
            StateRunnerError::QueueFull(e) => {write!(f, "Event Queue is full. Missed message: {}",e)}
        }

    }
}
impl Severity for StateRunnerError {
    fn get_severity(&self) -> ErrorSeverity {
        todo!()
    }
}

pub trait RunnableState {
    fn entry(&mut self, call_time: Microseconds<u64>);
    fn update(&mut self, iomanager: &mut dyn IOManager, call_time: Microseconds<u64>, event_queue: EventQueue) -> Result<(),StateRunnerError>;
    fn exit(&mut self, call_time: Microseconds<u64>);
}
