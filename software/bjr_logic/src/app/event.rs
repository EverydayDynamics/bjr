use core::fmt::{Display, Formatter};
use strum_macros::{EnumString, VariantNames};

#[derive(PartialEq, Copy, Clone, EnumString, VariantNames)]
pub enum GlobEvent {
    ButtonShortPress,
    ButtonLongPress,
    ErrorWithGracefulShutdown,
    ErrorWithImmediateShutdown,
    HomingFinished,
    InitFinished,
    EnterFeedforward,
    ExitFeedforward,
}
#[cfg(not(feature = "defmt"))]
impl Display for GlobEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            GlobEvent::ButtonShortPress => {write!(f,"GlobEvent Button Short Press")}
            GlobEvent::ButtonLongPress => {write!(f,"GlobEvent Button Long Press")}
            GlobEvent::ErrorWithGracefulShutdown => {write!(f,"GlobEvent Error With Graceful Shutdown")}
            GlobEvent::ErrorWithImmediateShutdown => {write!(f,"GlobEvent Error With Immediate Shutdown")}
            GlobEvent::HomingFinished => {write!(f,"GlobEvent Homing Finished")}
            GlobEvent::InitFinished => {write!(f,"GlobEvent Initialization Finished")}
            GlobEvent::EnterFeedforward => {write!(f,"GlobEvent Feed forward mode entry event")}
            GlobEvent::ExitFeedforward => {write!(f,"GlobEvent Feed forward mode exit event")}
        }
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for GlobEvent {
    fn format(&self, f: defmt::Formatter) {
        match self {
            GlobEvent::ButtonShortPress => {defmt::write!(f,"GlobEvent Button Short Press")}
            GlobEvent::ButtonLongPress => {defmt::write!(f,"GlobEvent Button Long Press")}
            GlobEvent::ErrorWithGracefulShutdown => {defmt::write!(f,"GlobEvent Error With Graceful Shutdown")}
            GlobEvent::ErrorWithImmediateShutdown => {defmt::write!(f,"GlobEvent Error With Immediate Shutdown")}
            GlobEvent::HomingFinished => {defmt::write!(f,"GlobEvent Homing Finished")}
            GlobEvent::InitFinished => {defmt::write!(f,"GlobEvent Initialization Finished")}
            GlobEvent::EnterFeedforward => {defmt::write!(f,"GlobEvent Feed forward mode entry event")}
            GlobEvent::ExitFeedforward => {defmt::write!(f,"GlobEvent Feed forward mode exit event")}
        }
    }
}

pub enum EventError {
    QueueFull(GlobEvent),
}
impl Display for EventError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            EventError::QueueFull(msg) => {write!(f,"EventError, queue full. lost message: {}",msg)}
        }
    }

}