use core::fmt::{Display, Formatter};
use bsp_traits::Logger;
use crate::app::event::GlobEvent;
use crate::app::event_queue::EventQueue;
use crate::str_to_display;
use crate::utils::DisplayStr;

#[derive(Clone, Copy, PartialEq)]
pub enum EventHandlerError {
    UnexpectedEvent(GlobEvent, State)
}
impl Display for EventHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            EventHandlerError::UnexpectedEvent(event, state) => {
                write!(f, "EventHandlerError Unexpected event ({}) received at state ({})", event, state)
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
impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let name= match self {
            State::Initializing => {"Initializing"}
            State::Homing => {"Homing"}
            State::RunningCenterHold => {"RunningCenterHold"}
            State::RunningCircling => {"RunningCircling"}
            State::RunningTriangle => {"RunningTriangle"}
            State::FeedForward => {"FeedForward"}
            State::Deinit => {"Deinit"}
            State::Off => {"Off"}
            State::Error => {"Error"}
        };
        write!(f, "{}", name)
    }
}
enum EventResponse {
    Ignore,
    Unexpected,
    NewState(State),
}

pub struct EventHandler {
    event_queue: EventQueue,
    state: State,
}
impl EventHandler {
    pub fn new(event_queue: EventQueue) -> Self {
        EventHandler {
            event_queue,
            state: State::Off,
        }
    }
    pub fn handle_events(&mut self, logger_device: &mut dyn Logger) -> Result<State, EventHandlerError>{
        while let Some(event) = self.event_queue.dequeue() {
            logger_device.info(&str_to_display!("Event received: {}", event));
            let response :EventResponse = match self.state {
                State::Initializing => {
                    match event {
                        GlobEvent::ButtonShortPress => {EventResponse::Ignore}
                        GlobEvent::ButtonLongPress => {EventResponse::Ignore}
                        GlobEvent::ErrorWithGracefulShutdown => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithImmediateShutdown => {EventResponse::NewState(State::Error)}
                        GlobEvent::HomingFinished => {EventResponse::NewState(State::RunningCenterHold)}
                        GlobEvent::InitFinished => {EventResponse::NewState(State::Homing)}
                        GlobEvent::EnterFeedforward => {EventResponse::Ignore}
                        GlobEvent::ExitFeedforward => {EventResponse::Ignore}
                    }
                }
                State::Homing => {
                    match event {
                        GlobEvent::ButtonShortPress => {EventResponse::Ignore}
                        GlobEvent::ButtonLongPress => {EventResponse::Ignore}
                        GlobEvent::ErrorWithGracefulShutdown => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithImmediateShutdown => {EventResponse::NewState(State::Error)}
                        GlobEvent::HomingFinished => {EventResponse::NewState(State::RunningCenterHold)}
                        GlobEvent::InitFinished => {EventResponse::Unexpected}
                        GlobEvent::EnterFeedforward => {EventResponse::Ignore}
                        GlobEvent::ExitFeedforward => {EventResponse::Ignore}
                    }
                }
                State::RunningCenterHold => {
                    match event {
                        GlobEvent::ButtonShortPress => {EventResponse::NewState(State::RunningCircling)}
                        GlobEvent::ButtonLongPress => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithGracefulShutdown => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithImmediateShutdown => {EventResponse::NewState(State::Error)}
                        GlobEvent::HomingFinished => {EventResponse::Unexpected}
                        GlobEvent::InitFinished => {EventResponse::Unexpected}
                        GlobEvent::EnterFeedforward => {EventResponse::NewState(State::FeedForward)}
                        GlobEvent::ExitFeedforward => {EventResponse::Ignore}
                    }
                }
                State::RunningCircling => {
                    match event {
                        GlobEvent::ButtonShortPress => {EventResponse::NewState(State::RunningTriangle)}
                        GlobEvent::ButtonLongPress => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithGracefulShutdown => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithImmediateShutdown => {EventResponse::NewState(State::Error)}
                        GlobEvent::HomingFinished => {EventResponse::Unexpected}
                        GlobEvent::InitFinished => {EventResponse::Unexpected}
                        GlobEvent::EnterFeedforward => {EventResponse::NewState(State::FeedForward)}
                        GlobEvent::ExitFeedforward => {EventResponse::Ignore}
                    }

                }
                State::RunningTriangle => {
                    match event {
                        GlobEvent::ButtonShortPress => {EventResponse::NewState(State::RunningCenterHold)}
                        GlobEvent::ButtonLongPress => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithGracefulShutdown => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithImmediateShutdown => {EventResponse::NewState(State::Error)}
                        GlobEvent::HomingFinished => {EventResponse::Unexpected}
                        GlobEvent::InitFinished => {EventResponse::Unexpected}
                        GlobEvent::EnterFeedforward => {EventResponse::NewState(State::FeedForward)}
                        GlobEvent::ExitFeedforward => {EventResponse::Ignore}
                    }
                }
                State::Deinit => {
                    match event {
                        GlobEvent::ButtonShortPress => {EventResponse::Ignore}
                        GlobEvent::ButtonLongPress => {EventResponse::Ignore}
                        GlobEvent::ErrorWithGracefulShutdown => {EventResponse::Ignore}
                        GlobEvent::ErrorWithImmediateShutdown => {EventResponse::NewState(State::Error)}
                        GlobEvent::HomingFinished => {EventResponse::Unexpected}
                        GlobEvent::InitFinished => {EventResponse::Unexpected}
                        GlobEvent::EnterFeedforward => {EventResponse::Ignore}
                        GlobEvent::ExitFeedforward => {EventResponse::Ignore}
                    }

                }
                State::Off => {
                    match event {
                        GlobEvent::ButtonShortPress => {EventResponse::Ignore}
                        GlobEvent::ButtonLongPress => {EventResponse::NewState(State::Initializing)}
                        GlobEvent::ErrorWithGracefulShutdown => {EventResponse::Ignore}
                        GlobEvent::ErrorWithImmediateShutdown => {EventResponse::NewState(State::Error)}
                        GlobEvent::HomingFinished => {EventResponse::NewState(State::RunningCenterHold)}
                        GlobEvent::InitFinished => {EventResponse::Unexpected}
                        GlobEvent::EnterFeedforward => {EventResponse::Ignore}
                        GlobEvent::ExitFeedforward => {EventResponse::Ignore}
                    }

                }
                State::Error => {
                    match event {
                        GlobEvent::ButtonShortPress => {EventResponse::Ignore}
                        GlobEvent::ButtonLongPress => {EventResponse::Ignore}
                        GlobEvent::ErrorWithGracefulShutdown => {EventResponse::Ignore}
                        GlobEvent::ErrorWithImmediateShutdown => {EventResponse::Ignore}
                        GlobEvent::HomingFinished => {EventResponse::Ignore}
                        GlobEvent::InitFinished => {EventResponse::Ignore}
                        GlobEvent::EnterFeedforward => {EventResponse::Ignore}
                        GlobEvent::ExitFeedforward => {EventResponse::Ignore}
                    }

                }
                State::FeedForward => {
                    match event {
                        GlobEvent::ButtonShortPress => {EventResponse::Ignore}
                        GlobEvent::ButtonLongPress => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithGracefulShutdown => {EventResponse::NewState(State::Deinit)}
                        GlobEvent::ErrorWithImmediateShutdown => {EventResponse::NewState(State::Error)}
                        GlobEvent::HomingFinished => {EventResponse::Unexpected}
                        GlobEvent::InitFinished => {EventResponse::Unexpected}
                        GlobEvent::EnterFeedforward => {EventResponse::NewState(State::FeedForward)}
                        GlobEvent::ExitFeedforward => {EventResponse::NewState(State::RunningCenterHold)}
                    }
                }
            };
           match response{
               EventResponse::Ignore => {
                   //TODO
                   //logger_device.warn(&str_to_display!("Incoming Event ({}) ignored! current state: {}", event ,self.state))
               }
               EventResponse::Unexpected => {
                   return Err(EventHandlerError::UnexpectedEvent(event, self.state))
               }
               EventResponse::NewState(new_state) => {
                   //TODO
                   //logger_device.info(&str_to_display!("state change occured: ({})->({}) Due to event: {}", self.state, new_state, event));
                   self.state = new_state;
               }
           }
        }
        Ok(self.state)
    }
}