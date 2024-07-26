
#[derive(PartialEq, Debug)]
pub enum GlobEvent {
    ButtonShortPress,
    ButtonLongPress,
    ErrorWithGracefulShutdown,
    ErrorWithImmediateShutdown,
}
#[derive(Debug)]
pub enum EventError {
    QueueFull,
}