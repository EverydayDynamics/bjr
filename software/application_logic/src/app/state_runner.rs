use crate::app::consts::MOTOR_NUM;
use crate::app::control_primitives::{KinState, PlateState};
use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::app::io_manager::{IOManager, IOManagerError};
use crate::app::severity_trait::{ErrorSeverity, Severity};
use crate::app::telemetry_handler::TelemetryBuilder;
use crate::app::trim::trimming::TrimmerError;
use core::fmt::{Display, Formatter};
use device_traits::{LoggableMessage, Logger, MotorEnabler};
use embedded_time::duration::Microseconds;

#[derive(PartialEq, Copy, Clone)]
pub enum StateRunnerError {
    HomingIOError(IOManagerError),
    HomingOverrun,
    HomingLimistSWStuckAtSafePos,
    HomingUnexpectedStopGoingToSafePos,
    ShutdownUnexpectedStopGoingToSafePos,
    HomingInErrorState,
    QueueFull(GlobEvent),
    IOError(IOManagerError),
    TrimmingError(TrimmerError),
}
impl LoggableMessage for StateRunnerError {}
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
            StateRunnerError::TrimmingError(e) => {
                write!(f, "Trimming error: {}", e)
            }
            StateRunnerError::ShutdownUnexpectedStopGoingToSafePos => {
                write!(
                    f,
                    "Movement Unexpectedly stopped while going to safe position in shutdown mode"
                )
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
            StateRunnerError::TrimmingError(_) => ErrorSeverity::Report,
            StateRunnerError::ShutdownUnexpectedStopGoingToSafePos => {
                ErrorSeverity::ImmediateShutdown
            }
        }
    }
}
#[derive(Copy, Clone)]
pub struct CirclingParams {
    pub height: f32,
    pub angulation_angle: f32,
    pub angulation_time: f32,
}
#[derive(Copy, Clone)]
pub enum StateRunnerCommand {
    FeedForwardPlateCommand(PlateState),
    FeedForwardMotorCommand([KinState; MOTOR_NUM]),
    FeedForwardCircling(CirclingParams),
    DebugMotorTest(usize),
    TrimPlateAngle,
    MotionDemo,
    NoCommand,
}
impl StateRunnerCommand {
    pub fn has_command(&self) -> bool {
        match self {
            StateRunnerCommand::NoCommand => false,
            _ => true,
        }
    }
}
impl Display for StateRunnerCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            StateRunnerCommand::TrimPlateAngle => {
                writeln!(f, "Trim the plate angle")
            }
            StateRunnerCommand::FeedForwardPlateCommand(_) => {
                writeln!(f, "Plate command")
            }
            StateRunnerCommand::FeedForwardMotorCommand(_) => {
                writeln!(f, "Plate command")
            }
            StateRunnerCommand::FeedForwardCircling(_) => {
                writeln!(f, "Circling")
            }
            StateRunnerCommand::DebugMotorTest(_) => {
                writeln!(f, "Debug Motor")
            }
            StateRunnerCommand::NoCommand => {
                writeln!(f, "No Command")
            }
            StateRunnerCommand::MotionDemo => {
                writeln!(f, "Motion Demo")
            }
        }
    }
}
pub struct StateRunnerContext<'a, LOG> {
    pub iomanager: &'a mut dyn IOManager,
    pub call_time: Microseconds<u64>,
    pub event_queue: EventQueue,
    pub logger: &'a mut LOG,
    pub command: &'a StateRunnerCommand,
    pub telemetry_builder: &'a mut TelemetryBuilder,
    pub motor_enabler: &'a mut dyn MotorEnabler,
    pub last_plate_setpoint: PlateState,
}
pub trait RunnableState {
    fn entry<LOG: Logger>(&mut self, ctx: &mut StateRunnerContext<LOG>);
    fn update<LOG: Logger>(
        &mut self,
        ctx: &mut StateRunnerContext<LOG>,
    ) -> Result<(), StateRunnerError>;
    fn exit<LOG: Logger>(&mut self, ctx: &mut StateRunnerContext<LOG>);
}
