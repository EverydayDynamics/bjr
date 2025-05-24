use core::fmt::{Display, Formatter};
use strum_macros::{EnumString, VariantNames};
use device_traits::LoggableMessage;

#[derive(PartialEq, Copy, Clone, EnumString, VariantNames)]
pub enum GlobEvent {
    ButtonShortPress,
    ButtonLongPress,
    ButtonDoublePress,
    ErrorWithGracefulShutdown,
    ErrorWithImmediateShutdown,
    HomingFinished,
    InitFinished,
    EnterFeedforward,
    ExitFeedforward,
}
impl LoggableMessage for GlobEvent {}
impl Display for GlobEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            GlobEvent::ButtonShortPress => {
                write!(f, "GlobEvent Button Short Press")
            }
            GlobEvent::ButtonLongPress => {
                write!(f, "GlobEvent Button Long Press")
            }
            GlobEvent::ErrorWithGracefulShutdown => {
                write!(f, "GlobEvent Error With Graceful Shutdown")
            }
            GlobEvent::ErrorWithImmediateShutdown => {
                write!(f, "GlobEvent Error With Immediate Shutdown")
            }
            GlobEvent::HomingFinished => {
                write!(f, "GlobEvent Homing Finished")
            }
            GlobEvent::InitFinished => {
                write!(f, "GlobEvent Initialization Finished")
            }
            GlobEvent::EnterFeedforward => {
                write!(f, "GlobEvent Feed forward mode entry event")
            }
            GlobEvent::ExitFeedforward => {
                write!(f, "GlobEvent Feed forward mode exit event")
            }
            GlobEvent::ButtonDoublePress => {
                write!(f, "GlobEvent Button Double Press")
            }
        }
    }
}

pub enum EventError {
    QueueFull(GlobEvent),
}
impl LoggableMessage for EventError {}
#[cfg(not(feature = "defmt"))]
impl Display for EventError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            EventError::QueueFull(msg) => {
                write!(f, "EventError, queue full. lost message: {}", msg)
            }
        }
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for  EventError{
    fn format(&self, f: defmt::Formatter) {
        match self {
            EventError::QueueFull(msg) => {
                defmt::write!(f, "EventError, queue full. lost message: {}", msg)
            }
        }
    }
}
