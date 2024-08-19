use crate::app::event::{EventError, GlobEvent};
use crate::app::severity_trait::{ErrorSeverity, Severity};
use crate::str_to_display;
use crate::utils::DisplayStr;
use bsp_traits::{Logger, MotorEnabler};
use core::fmt::Display;
use heapless::mpmc::Q8;

// Define the ErrorHandler struct
pub struct ErrorHandler {
    event_queue: &'static Q8<GlobEvent>,
}

impl ErrorHandler {
    pub fn new(event_queue: &'static Q8<GlobEvent>) -> Self {
        ErrorHandler { event_queue }
    }
}
impl ErrorHandler {
    pub fn panic<T: Display, LOG: Logger>(
        &mut self,
        error: T,
        motor_enabler: &mut dyn MotorEnabler,
        log_device: &mut LOG,
    ) {
        motor_enabler.set_enable(false);
        log_device.error(&error);
        panic!();
    }
    pub fn handle_error<ERR: Severity + Display, LOG: Logger>(
        &mut self,
        motor_enabler: &mut dyn MotorEnabler,
        log_device: &mut LOG,
        error: ERR,
    ) {
        match error.get_severity() {
            _ => {
                log_device.error(&error);
            }
        }
        match error.get_severity() {
            ErrorSeverity::Panic => self.panic(error, motor_enabler, log_device),
            ErrorSeverity::ImmediateShutdown => {
                motor_enabler.set_enable(false);
                let event = GlobEvent::ErrorWithImmediateShutdown;
                if self.event_queue.enqueue(event).is_err() {
                    self.panic(EventError::QueueFull(event), motor_enabler, log_device);
                }
            }
            ErrorSeverity::GracefulShutdown => {
                let event = GlobEvent::ErrorWithGracefulShutdown;
                if self.event_queue.enqueue(event).is_err() {
                    self.panic(event, motor_enabler, log_device);
                }
            }

            ErrorSeverity::Ignore => {
                log_device.warn(&str_to_display!("Ignored error: {}", error));
            }
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::{Display, Formatter};
    use mockall::mock;
    use mockall::predicate::*;
    use std::fmt::Arguments;
    use std::panic::AssertUnwindSafe;

    mock! {
        pub MotorEnabler {}
        impl MotorEnabler for MotorEnabler {
            fn set_enable(&mut self, enabled: bool);
        }
    }
    #[derive(Debug, Copy, Clone)]
    pub enum TestError {
        Panic,
        ImmediateShutdown,
        GracefulShutdown,
        Report,
        Ignore,
    }
    impl Display for TestError {
        fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
            write!(f, "{:?}", self)
        }
    }
    impl Severity for TestError {
        fn get_severity(&self) -> ErrorSeverity {
            match self {
                TestError::Panic => ErrorSeverity::Panic,
                TestError::ImmediateShutdown => ErrorSeverity::ImmediateShutdown,
                TestError::GracefulShutdown => ErrorSeverity::GracefulShutdown,
                TestError::Report => ErrorSeverity::Report,
                TestError::Ignore => ErrorSeverity::Ignore,
            }
        }
    }
    pub struct TestLogger {}
    impl Logger for TestLogger {
        fn trace(&mut self, message: &dyn Display) {}

        fn debug(&mut self, message: &dyn Display) {}

        fn info(&mut self, message: &dyn Display) {}

        fn warn(&mut self, message: &dyn Display) {}

        fn error(&mut self, message: &dyn Display) {}
    }

    #[test]
    fn test_error_handler_ignore() {
        static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
        let mut mock_motor_enabler = MockMotorEnabler::new();
        let mut mock_logger = TestLogger {};
        let mut test_error_handler = ErrorHandler::new(&EVENT_QUEUE);
        test_error_handler.handle_error(
            &mut mock_motor_enabler,
            &mut mock_logger,
            TestError::Ignore,
        );
    }
    #[test]
    fn test_error_handler_report() {
        static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
        let mut mock_motor_enabler = MockMotorEnabler::new();
        let mut mock_logger = TestLogger {};
        let mut test_error_handler = ErrorHandler::new(&EVENT_QUEUE);
        test_error_handler.handle_error(
            &mut mock_motor_enabler,
            &mut mock_logger,
            TestError::Report,
        );
        assert!(EVENT_QUEUE.dequeue() == None);
    }
    #[test]
    fn test_error_handler_graceful_shutdown() {
        static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
        let mut mock_motor_enabler = MockMotorEnabler::new();
        let mut mock_logger = TestLogger {};
        let mut test_error_handler = ErrorHandler::new(&EVENT_QUEUE);
        test_error_handler.handle_error(
            &mut mock_motor_enabler,
            &mut mock_logger,
            TestError::GracefulShutdown,
        );
        assert!(EVENT_QUEUE.dequeue() == Some(GlobEvent::ErrorWithGracefulShutdown));
        assert!(EVENT_QUEUE.dequeue() == None);
    }

    #[test]
    fn test_error_handler_immediate_shutdown() {
        static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
        let mut mock_motor_enabler = MockMotorEnabler::new();
        mock_motor_enabler
            .expect_set_enable()
            .with(eq(false))
            .times(1)
            .return_const(());

        let mut mock_logger = TestLogger {};
        let mut test_error_handler = ErrorHandler::new(&EVENT_QUEUE);
        test_error_handler.handle_error(
            &mut mock_motor_enabler,
            &mut mock_logger,
            TestError::ImmediateShutdown,
        );
        assert!(EVENT_QUEUE.dequeue() == Some(GlobEvent::ErrorWithImmediateShutdown));
        assert!(EVENT_QUEUE.dequeue() == None);
    }
}
