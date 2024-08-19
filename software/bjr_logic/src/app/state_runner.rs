use crate::app::control_primitives::PlateState;
use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::{IOManager, IOManagerError};
use crate::app::severity_trait::{ErrorSeverity, Severity};
use bsp_traits::{Logger, MotorEnabler};
use core::fmt::{Display, Formatter};
use embedded_time::duration::Microseconds;

#[derive(PartialEq, Copy, Clone)]
pub enum StateRunnerError {
    HomingIOError(IOManagerError),
    HomingOverrun,
    HomingLimistSWStuckAtSafePos,
    HomingUnexpectedStopGoingToSafePos,
    HomingInErrorState,
    QueueFull(GlobEvent),
    IOError(IOManagerError),
}
impl Display for StateRunnerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            StateRunnerError::HomingIOError(e) => {
                write!(f, "Homing IO error: {}", e)
            }
            StateRunnerError::HomingOverrun => {
                write!(f, "Homing Overrun")
            }
            StateRunnerError::HomingLimistSWStuckAtSafePos => {
                write!(f, "Homing Limit switch stuck at safet position")
            }
            StateRunnerError::HomingUnexpectedStopGoingToSafePos => {
                write!(
                    f,
                    "Homing Unexpectedly stopped while going to safe position"
                )
            }
            StateRunnerError::HomingInErrorState => {
                write!(f, "Homing is in error state")
            }
            StateRunnerError::QueueFull(e) => {
                write!(f, "Event Queue is full. Missed message: {}", e)
            }
            StateRunnerError::IOError(e) => {
                write!(f, "IO error: {}", e)
            }
        }
    }
}
impl Severity for StateRunnerError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            StateRunnerError::HomingIOError(_) => ErrorSeverity::ImmediateShutdown,
            StateRunnerError::HomingOverrun => ErrorSeverity::ImmediateShutdown,
            StateRunnerError::HomingLimistSWStuckAtSafePos => ErrorSeverity::ImmediateShutdown,
            StateRunnerError::HomingUnexpectedStopGoingToSafePos => {
                ErrorSeverity::ImmediateShutdown
            }
            StateRunnerError::HomingInErrorState => ErrorSeverity::ImmediateShutdown,
            StateRunnerError::QueueFull(_) => ErrorSeverity::Panic,
            StateRunnerError::IOError(_) => ErrorSeverity::ImmediateShutdown,
        }
    }
}
#[derive(Copy, Clone)]
pub enum StateRunnerCommand {
    FeedForwardCommand(PlateState),
    NoCommand,
}
pub trait RunnableState {
    fn entry(
        &mut self,
        call_time: Microseconds<u64>,
        motor_enabler: &mut dyn MotorEnabler,
        logger: &dyn Logger,
    );
    fn update(
        &mut self,
        iomanager: &mut dyn IOManager,
        call_time: Microseconds<u64>,
        event_queue: EventQueue,
        logger: &mut dyn Logger,
        command: &StateRunnerCommand,
    ) -> Result<(), StateRunnerError>;
    fn exit(&mut self, call_time: Microseconds<u64>, logger: &dyn Logger);
}
