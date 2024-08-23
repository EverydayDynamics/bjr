use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use bsp_traits::{LoggableMessage, Logger};
use core::fmt::{Display, Formatter};
#[cfg(feature = "defmt")]
use defmt;

#[derive(Clone, Copy, PartialEq)]
pub enum EventHandlerError {
    UnexpectedEvent(GlobEvent, State),
}
#[cfg(not(feature = "defmt"))]
impl Display for EventHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            EventHandlerError::UnexpectedEvent(event, state) => {
                write!(
                    f,
                    "EventHandlerError Unexpected event ({}) received at state ({})",
                    event, state
                )
            }
        }
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for EventHandlerError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            EventHandlerError::UnexpectedEvent(event, state) => {
                defmt::write!(
                    f,
                    "EventHandlerError Unexpected event ({}) received at state ({})",
                    event, state
                )
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum State {
    Initializing,
    Homing,
    RunningCenterHold,
    RunningCircling,
    RunningTriangle,
    FeedForward,
    Deinit,
    Error,
    Off,
}
impl LoggableMessage for State {}
#[cfg(not(feature = "defmt"))]
impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let name = match self {
            State::Initializing => "Initializing",
            State::Homing => "Homing",
            State::RunningCenterHold => "RunningCenterHold",
            State::RunningCircling => "RunningCircling",
            State::RunningTriangle => "RunningTriangle",
            State::FeedForward => "FeedForward",
            State::Deinit => "Deinit",
            State::Off => "Off",
            State::Error => "Error",
        };
        write!(f, "{}", name)
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for State {
    fn format(&self, f: defmt::Formatter) {
        let name = match self {
            State::Initializing => "Initializing",
            State::Homing => "Homing",
            State::RunningCenterHold => "RunningCenterHold",
            State::RunningCircling => "RunningCircling",
            State::RunningTriangle => "RunningTriangle",
            State::FeedForward => "FeedForward",
            State::Deinit => "Deinit",
            State::Off => "Off",
            State::Error => "Error",
        };
        defmt::write!(f, "{}", name)
    }
}
struct EventReceivedMessage(GlobEvent);
#[cfg(not(feature = "defmt"))]
impl Display for  EventReceivedMessage{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f,"Event received: {}", self.0)
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for  EventReceivedMessage{
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f,"Event received: {}", self.0)
    }
}
impl LoggableMessage for EventReceivedMessage {}

struct StateChangeMessage(State, State, GlobEvent);

#[cfg(not(feature = "defmt"))]
impl Display for  StateChangeMessage{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f,"state change occured: ({})->({}) Due to event: {}", self.0, self.1, self.2)
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for  StateChangeMessage{
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f,"state change occured: ({})->({}) Due to event: {}", self.0, self.1, self.2)
    }
}
impl LoggableMessage for StateChangeMessage {}
enum EventResponse {
    Ignore,
    Unexpected,
    NewState(State),
}

pub struct EventHandler {
    event_queue: EventQueue,
    state: State,
}
struct IgnoredEventWarning(GlobEvent, State);
impl LoggableMessage for IgnoredEventWarning {}
#[cfg(not(feature = "defmt"))]
impl Display for  IgnoredEventWarning{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f,"Incoming Event ({}) ignored! current state: {}", self.0 ,self.1 )
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for IgnoredEventWarning {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f,"Incoming Event ({}) ignored! current state: {}", self.0 ,self.1 )
    }
}
impl EventHandler {
    pub fn new(event_queue: EventQueue) -> Self {
        EventHandler {
            event_queue,
            state: State::Off,
        }
    }
    pub fn handle_events<LOG: Logger>(
        &mut self,
        logger_device: &mut LOG,
    ) -> Result<State, EventHandlerError> {
        while let Some(event) = self.event_queue.dequeue() {
            logger_device.info(EventReceivedMessage(event));
            let response: EventResponse = match self.state {
                State::Initializing => match event {
                    GlobEvent::ButtonShortPress => EventResponse::Ignore,
                    GlobEvent::ButtonLongPress => EventResponse::Ignore,
                    GlobEvent::ErrorWithGracefulShutdown => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithImmediateShutdown => EventResponse::NewState(State::Error),
                    GlobEvent::HomingFinished => EventResponse::NewState(State::RunningCenterHold),
                    GlobEvent::InitFinished => EventResponse::NewState(State::Homing),
                    GlobEvent::EnterFeedforward => EventResponse::Ignore,
                    GlobEvent::ExitFeedforward => EventResponse::Ignore,
                },
                State::Homing => match event {
                    GlobEvent::ButtonShortPress => EventResponse::Ignore,
                    GlobEvent::ButtonLongPress => EventResponse::Ignore,
                    GlobEvent::ErrorWithGracefulShutdown => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithImmediateShutdown => EventResponse::NewState(State::Error),
                    GlobEvent::HomingFinished => EventResponse::NewState(State::RunningCenterHold),
                    GlobEvent::InitFinished => EventResponse::Unexpected,
                    GlobEvent::EnterFeedforward => EventResponse::Ignore,
                    GlobEvent::ExitFeedforward => EventResponse::Ignore,
                },
                State::RunningCenterHold => match event {
                    GlobEvent::ButtonShortPress => EventResponse::NewState(State::RunningCircling),
                    GlobEvent::ButtonLongPress => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithGracefulShutdown => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithImmediateShutdown => EventResponse::NewState(State::Error),
                    GlobEvent::HomingFinished => EventResponse::Unexpected,
                    GlobEvent::InitFinished => EventResponse::Unexpected,
                    GlobEvent::EnterFeedforward => EventResponse::NewState(State::FeedForward),
                    GlobEvent::ExitFeedforward => EventResponse::Ignore,
                },
                State::RunningCircling => match event {
                    GlobEvent::ButtonShortPress => EventResponse::NewState(State::RunningTriangle),
                    GlobEvent::ButtonLongPress => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithGracefulShutdown => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithImmediateShutdown => EventResponse::NewState(State::Error),
                    GlobEvent::HomingFinished => EventResponse::Unexpected,
                    GlobEvent::InitFinished => EventResponse::Unexpected,
                    GlobEvent::EnterFeedforward => EventResponse::NewState(State::FeedForward),
                    GlobEvent::ExitFeedforward => EventResponse::Ignore,
                },
                State::RunningTriangle => match event {
                    GlobEvent::ButtonShortPress => {
                        EventResponse::NewState(State::RunningCenterHold)
                    }
                    GlobEvent::ButtonLongPress => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithGracefulShutdown => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithImmediateShutdown => EventResponse::NewState(State::Error),
                    GlobEvent::HomingFinished => EventResponse::Unexpected,
                    GlobEvent::InitFinished => EventResponse::Unexpected,
                    GlobEvent::EnterFeedforward => EventResponse::NewState(State::FeedForward),
                    GlobEvent::ExitFeedforward => EventResponse::Ignore,
                },
                State::Deinit => match event {
                    GlobEvent::ButtonShortPress => EventResponse::Ignore,
                    GlobEvent::ButtonLongPress => EventResponse::Ignore,
                    GlobEvent::ErrorWithGracefulShutdown => EventResponse::Ignore,
                    GlobEvent::ErrorWithImmediateShutdown => EventResponse::NewState(State::Error),
                    GlobEvent::HomingFinished => EventResponse::Unexpected,
                    GlobEvent::InitFinished => EventResponse::Unexpected,
                    GlobEvent::EnterFeedforward => EventResponse::Ignore,
                    GlobEvent::ExitFeedforward => EventResponse::Ignore,
                },
                State::Off => match event {
                    GlobEvent::ButtonShortPress => EventResponse::Ignore,
                    GlobEvent::ButtonLongPress => EventResponse::NewState(State::Initializing),
                    GlobEvent::ErrorWithGracefulShutdown => EventResponse::Ignore,
                    GlobEvent::ErrorWithImmediateShutdown => EventResponse::NewState(State::Error),
                    GlobEvent::HomingFinished => EventResponse::NewState(State::RunningCenterHold),
                    GlobEvent::InitFinished => EventResponse::Unexpected,
                    GlobEvent::EnterFeedforward => EventResponse::Ignore,
                    GlobEvent::ExitFeedforward => EventResponse::Ignore,
                },
                State::Error => match event {
                    GlobEvent::ButtonShortPress => EventResponse::Ignore,
                    GlobEvent::ButtonLongPress => EventResponse::Ignore,
                    GlobEvent::ErrorWithGracefulShutdown => EventResponse::Ignore,
                    GlobEvent::ErrorWithImmediateShutdown => EventResponse::Ignore,
                    GlobEvent::HomingFinished => EventResponse::Ignore,
                    GlobEvent::InitFinished => EventResponse::Ignore,
                    GlobEvent::EnterFeedforward => EventResponse::Ignore,
                    GlobEvent::ExitFeedforward => EventResponse::Ignore,
                },
                State::FeedForward => match event {
                    GlobEvent::ButtonShortPress => EventResponse::Ignore,
                    GlobEvent::ButtonLongPress => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithGracefulShutdown => EventResponse::NewState(State::Deinit),
                    GlobEvent::ErrorWithImmediateShutdown => EventResponse::NewState(State::Error),
                    GlobEvent::HomingFinished => EventResponse::Unexpected,
                    GlobEvent::InitFinished => EventResponse::Unexpected,
                    GlobEvent::EnterFeedforward => EventResponse::NewState(State::FeedForward),
                    GlobEvent::ExitFeedforward => EventResponse::NewState(State::RunningCenterHold),
                },
            };
            match response {
                EventResponse::Ignore => {
                    logger_device.warn(IgnoredEventWarning(event ,self.state));
                }
                EventResponse::Unexpected => {
                    return Err(EventHandlerError::UnexpectedEvent(event, self.state))
                }
                EventResponse::NewState(new_state) => {
                    logger_device.info(StateChangeMessage(self.state, new_state, event));
                    self.state = new_state;
                }
            }
        }
        Ok(self.state)
    }
}
