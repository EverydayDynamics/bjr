use crate::app::event::GlobEvent;
use crate::app::parameter_manager::{parameter_manager, DoublePressThresholdMs, LongPressThresholdMs, ParameterManager};
use crate::app::severity_trait::{ErrorSeverity, Severity};
use device_traits::{Button, LoggableMessage};
use core::fmt::{Display, Formatter};
use embedded_time::duration::*;
use heapless::mpmc::Q8;
use crate::app::button_handler::ButtonHandlerError::QueueFull;

#[derive(PartialEq)]
pub enum ButtonHandlerError {
    QueueFull(GlobEvent),
}
impl LoggableMessage for ButtonHandlerError {}
#[cfg(not(feature = "defmt"))]
impl Display for ButtonHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ButtonHandlerError::QueueFull(ge) => {
                write!(f, "ButtonHandlerError Queue Full. Dropped msg: {}", ge)
            }
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for ButtonHandlerError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            ButtonHandlerError::QueueFull(ge) => {
                defmt::write!(f, "ButtonHandlerError Queue Full. Dropped msg: {}", ge)
            }
        }
    }
}
impl Severity for ButtonHandlerError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            ButtonHandlerError::QueueFull(_) => ErrorSeverity::Panic,
        }
    }
}
enum ButtonHandlerState{
    NoPress,
    FirstDown(Microseconds<u64>),
    FirstUp(Microseconds<u64>),
    SecondDown(Microseconds<u64>),
}
pub struct ButtonHandler<BTN> {
    button: BTN,
    event_handler: &'static Q8<GlobEvent>,
    state: ButtonHandlerState,
}

impl<BTN: Button> ButtonHandler<BTN> {
    pub fn new(button: BTN, event_handler: &'static Q8<GlobEvent>) -> Self {
        ButtonHandler {
            button,
            event_handler,
            state: ButtonHandlerState::NoPress,
        }
    }

    pub fn update(&mut self, call_time: Microseconds<u64>) -> Result<(), ButtonHandlerError> {
        let currently_pressed = self.button.is_pressed();
        let event_to_send:Option<GlobEvent> = match self.state {
            ButtonHandlerState::NoPress => {
                if currently_pressed {
                    self.state = ButtonHandlerState::FirstDown(call_time);
                }
                None
            }
            ButtonHandlerState::FirstDown(fdt) => {
                if !currently_pressed {
                    let longpress_thrs: Microseconds<u64>= Microseconds::new(
                        (parameter_manager().get::<LongPressThresholdMs>()*1000) as u64);
                    if (call_time - fdt) > longpress_thrs {
                        self.state = ButtonHandlerState::NoPress;
                        Some(GlobEvent::ButtonLongPress)
                    } else {
                        self.state = ButtonHandlerState::FirstUp(call_time);
                        None
                    }
                }else {
                    None
                }
            }
            ButtonHandlerState::FirstUp(fut) => {
                let doublepress_thrs: Microseconds<u64>= Microseconds::new(
                    (parameter_manager().get::<DoublePressThresholdMs>()*1000) as u64);
                if (call_time-fut) > doublepress_thrs {
                    self.state = ButtonHandlerState::NoPress;
                    Some(GlobEvent::ButtonShortPress)
                } else {
                    if currently_pressed {
                        self.state = ButtonHandlerState::SecondDown(call_time);
                    }
                    None
                }
            }
            ButtonHandlerState::SecondDown(sdt) => {
                if !currently_pressed {
                    let longpress_thrs: Microseconds<u64>= Microseconds::new(
                        (parameter_manager().get::<LongPressThresholdMs>()*1000) as u64);
                    if (call_time - sdt) > longpress_thrs {
                        self.state = ButtonHandlerState::NoPress;
                        Some(GlobEvent::ButtonLongPress)
                    } else {
                        self.state = ButtonHandlerState::NoPress;
                        Some(GlobEvent::ButtonDoublePress)
                    }
                } else {
                   None
                }
            }
        };
        if let Some(event_to_send) = event_to_send {
            self.event_handler.enqueue(event_to_send).map_err(|_| QueueFull(event_to_send))?;
        }
        Ok(())
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::event_queue::drain_event_queue;
    use mockall::mock;

    mock! {
        pub Button {}
        impl Button for Button {
            fn is_pressed(&mut self) -> bool;
        }
    }
    static EVENT_QUEUE_1: Q8<GlobEvent> = Q8::new();
    static EVENT_QUEUE_2: Q8<GlobEvent> = Q8::new();
    static EVENT_QUEUE_3: Q8<GlobEvent> = Q8::new();
    #[test]
    fn test_button_handler_nopress() {
        let run_count = 10u64;
        let call_rate = Microseconds::<u64>(1_000_u64);
        let mut mock_button = MockButton::new();
        mock_button
            .expect_is_pressed()
            .times(run_count.clone() as usize)
            .return_const(false);
        let mut test_button_handler = ButtonHandler::new(mock_button, &EVENT_QUEUE_1);
        for run_num in 1..(run_count + 1) {
            let call_time = call_rate * run_num;
            assert!(test_button_handler.update(call_time)==Ok(()));
        }
        assert!(EVENT_QUEUE_1.dequeue() == None);
    }
    #[test]
    fn test_button_handler_shortpress() {
        drain_event_queue();
        let run_count = 10u64;
        let call_rate = Microseconds::<u64>(10_000_u64);
        let mut mock_button = MockButton::new();
        mock_button.expect_is_pressed().times(1).return_const(false);
        mock_button.expect_is_pressed().times(5).return_const(true);
        mock_button.expect_is_pressed().times(4).return_const(false);
        let mut test_button_handler = ButtonHandler::new(mock_button, &EVENT_QUEUE_2);
        for run_num in 1..(run_count + 1) {
            let call_time = call_rate * run_num;
            assert!(test_button_handler.update(call_time)==Ok(()));
        }
        assert!(EVENT_QUEUE_2.dequeue() == Some(GlobEvent::ButtonShortPress));
        assert!(EVENT_QUEUE_2.dequeue() == None);
        drain_event_queue();
    }

    #[test]
    fn test_button_handler_longnshort_press() {
        drain_event_queue();
        let run_count = 200u64;
        let call_rate = Microseconds::<u64>(10_000_u64);
        let mut mock_button = MockButton::new();
        mock_button
            .expect_is_pressed()
            .times(101)
            .return_const(true);
        mock_button
            .expect_is_pressed()
            .times(20)
            .return_const(false);
        mock_button.expect_is_pressed().times(10).return_const(true);
        mock_button
            .expect_is_pressed()
            .times(69)
            .return_const(false);
        let mut test_button_handler = ButtonHandler::new(mock_button, &EVENT_QUEUE_3);
        for run_num in 1..(run_count + 1) {
            let call_time = call_rate * run_num;
            let result = test_button_handler.update(call_time);
            assert!(result == Ok(()));
        }
        assert!(EVENT_QUEUE_3.dequeue() == Some(GlobEvent::ButtonLongPress));
        assert!(EVENT_QUEUE_3.dequeue() == Some(GlobEvent::ButtonShortPress));
        assert!(EVENT_QUEUE_3.dequeue() == None);
        drain_event_queue();
    }
}
