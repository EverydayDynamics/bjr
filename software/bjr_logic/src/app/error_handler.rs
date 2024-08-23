use crate::app::event::{EventError, GlobEvent};
use crate::app::severity_trait::{ErrorSeverity, Severity};
use bsp_traits::{LoggableMessage, Logger, MotorEnabler};
use core::fmt::{Display, Formatter};
use heapless::mpmc::Q8;

pub struct ErrorHandler {
    event_queue: &'static Q8<GlobEvent>,
}
struct ErrorIgnoredMessage<ERR:LoggableMessage>(ERR);
impl<ERR: LoggableMessage> LoggableMessage for ErrorIgnoredMessage<ERR> {}
#[cfg(not(feature = "defmt"))]
impl<ERR: LoggableMessage> Display for ErrorIgnoredMessage<ERR> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Ignored Error: {}", self.0)
    }
}


#[cfg(feature = "defmt")]
impl<ERR: LoggableMessage> defmt::Format for  ErrorIgnoredMessage<ERR> {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Ignored Error: {}", self.0);
    }
}
impl ErrorHandler {
    pub fn new(event_queue: &'static Q8<GlobEvent>) -> Self {
        ErrorHandler { event_queue }
    }
}
impl ErrorHandler {
    pub fn panic<LOG: Logger>(
        &mut self,
        motor_enabler: &mut dyn MotorEnabler,
        _log_device: &mut LOG,
    ) {
        motor_enabler.set_enable(false);
        panic!();
    }
    pub fn handle_error<ERR: Severity + LoggableMessage, LOG: Logger>(
        &mut self,
        motor_enabler: &mut dyn MotorEnabler,
        log_device: &mut LOG,
        error: ERR,
    ) {
        let severity = error.get_severity();
        match severity {
            ErrorSeverity::Ignore => {
                log_device.warn(ErrorIgnoredMessage(error));
            }
            _ => {
                log_device.error(error);
            }
        }
        match severity {
            ErrorSeverity::Panic => self.panic(motor_enabler, log_device),
            ErrorSeverity::ImmediateShutdown => {
                motor_enabler.set_enable(false);
                let event = GlobEvent::ErrorWithImmediateShutdown;
                if self.event_queue.enqueue(event).is_err() {
                    let new_error = EventError::QueueFull(event);
                    log_device.error(new_error);
                    self.panic(motor_enabler, log_device);
                }
            }
            ErrorSeverity::GracefulShutdown => {
                let event = GlobEvent::ErrorWithGracefulShutdown;
                if self.event_queue.enqueue(event).is_err() {
                    let new_error = EventError::QueueFull(event);
                    log_device.error(new_error);
                    self.panic(motor_enabler, log_device);
                }
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
        fn trace<MSG: LoggableMessage>(&mut self, message: MSG) {
        }

        fn debug<MSG: LoggableMessage>(&mut self, message: MSG) {
        }

        fn info<MSG: LoggableMessage>(&mut self, message: MSG) {
        }

        fn warn<MSG: LoggableMessage>(&mut self, message: MSG) {
        }

        fn error<MSG: LoggableMessage>(&mut self, message: MSG) {
        }
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
