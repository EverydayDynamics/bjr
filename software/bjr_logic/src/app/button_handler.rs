use core::fmt::{Display, Formatter};
use crate::app::parameter_manager::{parameter_manager, LongPressThresholdMs};
use bsp_traits::Button;
use crate::app::event::GlobEvent;
use embedded_time::{duration::*};
use heapless::mpmc::Q8;
use crate::app::severity_trait::{ErrorSeverity, Severity};

#[derive(PartialEq)]
pub enum ButtonHandlerError {
    QueueFull(GlobEvent)
}
impl Display for ButtonHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ButtonHandlerError::QueueFull(ge) => {write!(f,"ButtonHandlerError Queue Full. Dropped msg: {}", ge)}
        }
    }
}
impl Severity for ButtonHandlerError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            ButtonHandlerError::QueueFull(_) => {ErrorSeverity::Panic}
        }
    }
}

pub struct ButtonHandler<BTN> {
    button: BTN,
    event_handler: &'static Q8<GlobEvent>,
    last_state: bool,
    press_start: Microseconds<u64>,
}

impl<BTN: Button> ButtonHandler<BTN> {
    pub fn new(button: BTN, event_handler: &'static Q8<GlobEvent>) -> Self {
        ButtonHandler {
            button,
            event_handler,
            last_state: false,
            press_start: Microseconds::new(0u64),
        }
    }

    pub fn update(&mut self, call_time: Microseconds<u64>) -> Result<(),ButtonHandlerError>{
        let currently_pressed = self.button.is_pressed();
        if self.last_state != currently_pressed {
            // State change detected!
            if currently_pressed {
                // state changed to pressed
                self.press_start = call_time;
            } else {
                // state changed to released
                let press_duration = call_time - self.press_start;
                let event_to_send = if press_duration > Milliseconds::new(parameter_manager().get::<LongPressThresholdMs>()) {
                    GlobEvent::ButtonLongPress
                } else {
                    GlobEvent::ButtonShortPress
                };
                self.event_handler.enqueue(event_to_send).map_err(|e|ButtonHandlerError::QueueFull(e))?;
            }
        }
        self.last_state = currently_pressed;
        Ok(())
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use crate::app::event_queue::drain_event_queue;
    use crate::app::parameter_manager::parameter_manager;

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
        mock_button.expect_is_pressed()
            .times(run_count.clone()as usize)
            .return_const(false);
        let mut test_button_handler = ButtonHandler::new(mock_button, &EVENT_QUEUE_1);
        for run_num in 1..(run_count+1) {
            let call_time = call_rate*run_num;
            test_button_handler.update(call_time);
        }
        assert!(EVENT_QUEUE_1.dequeue()== None);
    }
    #[test]
    fn test_button_handler_shortpress() {
        drain_event_queue();
        let run_count = 10u64;
        let call_rate = Microseconds::<u64>(10_000_u64);
        let mut mock_button = MockButton::new();
        mock_button.expect_is_pressed()
            .times(1)
            .return_const(false);
        mock_button.expect_is_pressed()
            .times(5)
            .return_const(true);
        mock_button.expect_is_pressed()
            .times(4)
            .return_const(false);
        let mut test_button_handler = ButtonHandler::new(mock_button, &EVENT_QUEUE_2);
        for run_num in 1..(run_count+1) {
            let call_time = call_rate*run_num;
            test_button_handler.update(call_time);
        }
        assert!(EVENT_QUEUE_2.dequeue()== Some(GlobEvent::ButtonShortPress));
        assert!(EVENT_QUEUE_2.dequeue()== None);
        drain_event_queue();
    }

#[test]
fn test_button_handler_longnshort_press() {
    drain_event_queue();
    let run_count = 200u64;
    let call_rate = Microseconds::<u64>(10_000_u64);
    let mut mock_button = MockButton::new();
    mock_button.expect_is_pressed()
        .times(101)
        .return_const(true);
    mock_button.expect_is_pressed()
        .times(20)
        .return_const(false);
    mock_button.expect_is_pressed()
        .times(10)
        .return_const(true);
    mock_button.expect_is_pressed()
        .times(69)
       .return_const(false);
    let mut test_button_handler = ButtonHandler::new(mock_button, &EVENT_QUEUE_3);
    for run_num in 1..(run_count+1) {
        let call_time = call_rate*run_num;
        let result = test_button_handler.update(call_time);
        assert!(result == Ok(()));
    }
    assert!(EVENT_QUEUE_3.dequeue() == Some(GlobEvent::ButtonLongPress));
    assert!(EVENT_QUEUE_3.dequeue() == Some(GlobEvent::ButtonShortPress));
    assert!(EVENT_QUEUE_3.dequeue() == None);
    drain_event_queue();
}
}
