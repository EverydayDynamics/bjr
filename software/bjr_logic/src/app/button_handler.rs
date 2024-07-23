use crate::app::parameter_manager::{get_parameter_manager, LongPressThresholdMs};
use bsp_traits::Button;
use crate::app::event::GlobEvent;
use embedded_time::{duration::*, rate::*};
use crate::app::event_queue::get_event_queue;
use crate::app::parameter_manager::ParameterManager;
// Define a trait for the button hardware interface

// Define the possible button events
pub struct ButtonHandler<'a> {
    button: &'a mut dyn Button,
    parameter_manager: &'static ParameterManager,
    last_state: bool,
    press_start: Microseconds<u64>,
}

impl<'a> ButtonHandler<'a> {
    pub fn new(button: &'a mut dyn Button) -> Self {
        ButtonHandler {
            button,
            parameter_manager: get_parameter_manager(),
            last_state: false,
            press_start: Microseconds::new(0u64),
        }
    }

    pub fn update(&mut self, call_time: Microseconds<u64>) {
        let currently_pressed = self.button.is_pressed();
        if self.last_state != currently_pressed {
            // State change detected!
            if (currently_pressed) {
                // state changed to pressed
                self.press_start = call_time;
            } else {
                // state changed to released
                let press_duration = call_time - self.press_start;
                let maram = self.parameter_manager.get::<LongPressThresholdMs>();
                println!("paramerter:{:?}", maram);
                let event_to_send = if press_duration > Milliseconds::new(1000u64) {
                    GlobEvent::ButtonLongPress
                } else {
                    GlobEvent::ButtonShortPress
                };
                get_event_queue().enqueue(event_to_send).unwrap();
            }
        }
        self.last_state = currently_pressed
    }
}

// Example usage
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use crate::app::event_queue::drain_event_queue;
    use crate::app::parameter_manager::get_parameter_manager;

    mock! {
        pub Button {}
        impl Button for Button {
            fn is_pressed(&mut self) -> bool;
        }
    }

    #[test]
    fn test_button_handler_nopress() {
        drain_event_queue();
        let run_count = 10u64;
        let call_rate = Microseconds::<u64>(1_000_u64);
        let mut mock_button = MockButton::new();
        mock_button.expect_is_pressed()
            .times(run_count.clone()as usize)
            .return_const(false);
        let mut test_button_handler = ButtonHandler::new(&mut mock_button);
        for run_num in 1..(run_count+1) {
            let call_time = call_rate*run_num;
            test_button_handler.update(call_time);
        }
        assert_eq!(get_event_queue().dequeue(), None);
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
        let mut test_button_handler = ButtonHandler::new(&mut mock_button);
        for run_num in 1..(run_count+1) {
            let call_time = call_rate*run_num;
            test_button_handler.update(call_time);
        }
        assert_eq!(get_event_queue().dequeue(), Some(GlobEvent::ButtonShortPress));
        assert_eq!(get_event_queue().dequeue(), None);
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
    let mut test_button_handler = ButtonHandler::new(&mut mock_button);
    for run_num in 1..(run_count+1) {
        let call_time = call_rate*run_num;
        test_button_handler.update(call_time);
    }
    assert_eq!(get_event_queue().dequeue(), Some(GlobEvent::ButtonLongPress));
    assert_eq!(get_event_queue().dequeue(), Some(GlobEvent::ButtonShortPress));
    assert_eq!(get_event_queue().dequeue(), None);
}
}
