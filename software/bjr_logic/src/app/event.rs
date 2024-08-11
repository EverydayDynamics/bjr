use core::fmt::{Display, Formatter};

#[derive(PartialEq, Copy, Clone)]
pub enum GlobEvent {
    ButtonShortPress,
    ButtonLongPress,
    ErrorWithGracefulShutdown,
    ErrorWithImmediateShutdown,
    HomingFinished,
    InitFinished,
}
impl Display for GlobEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            GlobEvent::ButtonShortPress => {write!(f,"GlobEvent Button Short Press")}
            GlobEvent::ButtonLongPress => {write!(f,"GlobEvent Button Long Press")}
            GlobEvent::ErrorWithGracefulShutdown => {write!(f,"GlobEvent Error With Graceful Shutdown")}
            GlobEvent::ErrorWithImmediateShutdown => {write!(f,"GlobEvent Error With Immediate Shutdown")}
            GlobEvent::HomingFinished => {write!(f,"GlobEvent Homing Finished")}
            GlobEvent::InitFinished => {write!(f,"GlobEvent Initialization Finished")}
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