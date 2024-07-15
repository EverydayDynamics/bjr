use crate::bsp::traits::Button;
use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;
// Define a trait for the button hardware interface

// Define the possible button events
pub struct ButtonHandler<'a> {
    button: &'a mut dyn Button,
    sender: &'static Q8<GlobEvent>,
    last_state: bool,
    press_duration: u32,
    long_press_threshold: u32,
}

impl<'a> ButtonHandler<'a> {
    pub fn new(button: &'a mut dyn Button, sender: &'static Q8<GlobEvent>) -> Self {
        ButtonHandler {
            button,
            sender,
            last_state: false,
            press_duration: 0,
            long_press_threshold: 1000, // 1 second, adjust as needed
        }
    }

    pub fn update(&mut self, call_time: u32) {
        let current_state = self.button.is_pressed();
    }
}

// Example usage
#[defmt_test::tests]
#[cfg(test)]
mod tests {
    use super::*;

    struct MockButton {
        pressed: bool,
    }

    impl Button for MockButton {
        fn is_pressed(&mut self) -> bool {
            self.pressed
        }
    }

    #[test]
    fn test_button_handler() {
        //let (tx, rx) = mpsc::channel();
        //let mock_button = Box::new(MockButton { pressed: false });
        //let mut handler = ButtonHandler::new(mock_button, tx);

        //// Simulate button press
        //handler.button = Box::new(MockButton { pressed: true });
        //handler.update(100);
        //assert_eq!(rx.try_recv(), Ok(ButtonEvent::Pressed));

        //// Simulate button hold
        //handler.update(900);
        //assert_eq!(rx.try_recv(), Ok(ButtonEvent::LongPress));

        //// Simulate button release
        //handler.button = Box::new(MockButton { pressed: false });
        //handler.update(100);
        //assert_eq!(rx.try_recv(), Ok(ButtonEvent::Released));
        //assert_eq!(rx.try_recv(), Ok(ButtonEvent::LongPress));
    }
}