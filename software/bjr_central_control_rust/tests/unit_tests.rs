#![no_main]
#![no_std]
use bjr::bsp::traits::Button;
use cortex_m_semihosting::debug;
use defmt_rtt as _; // global logger

use stm32f4xx_hal as _; // memory layout
use panic_probe as _;
use bjr as _;
// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked


/// Terminates the application and makes a semihosting-capable debug tool exit
/// with status code 0.
pub fn exit() -> ! {
    loop {
        debug::exit(debug::EXIT_SUCCESS);
    }
}

/// Hardfault handler.
///
/// Terminates the application and makes a semihosting-capable debug tool exit
/// with an error. This seems better than the default, which is to spin in a
/// loop.
#[defmt_test::tests]
#[cfg(test)]
mod button_handler_tests {
    use heapless::mpmc::Q8;
    use bjr::app::button_handler::ButtonHandler;
    use bjr::app::event::GlobEvent;
    use super::*;

    struct MockButton {
        pub pressed: bool,
    }

    impl Button for MockButton {
        fn is_pressed(&mut self) -> bool {
            self.pressed
        }
    }

    static EVENTQUEUE : Q8<GlobEvent> = Q8::new();
    #[test]
    fn test_button_handler() {
        let mut mock_button = MockButton{ pressed: false };
        let mut handler = ButtonHandler::new(&mut mock_button, &EVENTQUEUE);

        handler.update(100);
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
