use bsp_traits::{MotorEnabler, Logger};
use heapless::mpmc::Q8;
use crate::app::event::{EventError, GlobEvent};
use crate::app::severity_trait::{ErrorSeverity, Severity};

// Define the ErrorHandler struct
pub struct ErrorHandler<'a> {
    motor_enabler: &'a mut dyn MotorEnabler,
    log_device: &'a mut dyn Logger,
    event_queue: &'static Q8<GlobEvent>,
}

impl<'a> ErrorHandler<'a>{
    pub fn new(motor_enabler: &'a mut dyn MotorEnabler, log_device: &'a mut dyn Logger, event_queue: &'static Q8<GlobEvent>) -> Self {
        ErrorHandler{
            motor_enabler,
            log_device,
            event_queue,
        }
    }
}
impl ErrorHandler<'_>
{
    pub fn panic<T>(&mut self, error: T) {
        self.motor_enabler.set_enable(false);
        //Err::<(),T>(error).unwrap();
        panic!();
    }
    pub fn handle_error<ERR: Severity + core::fmt::Display>(&mut self, error:ERR) {
        match error.get_severity() {
            ErrorSeverity::Ignore => {}
            _ => {self.log_device.error(format_args!("{}",error));}
        }
        match error.get_severity() {
            ErrorSeverity::Panic => { self.panic(error) }
            ErrorSeverity::ImmediateShutdown => {
                self.motor_enabler.set_enable(false);
                if self.event_queue.enqueue(GlobEvent::ErrorWithImmediateShutdown).is_err() {
                    self.panic(EventError::QueueFull);
                }}
            ErrorSeverity::GracefulShutdown => {
                if self.event_queue.enqueue(GlobEvent::ErrorWithGracefulShutdown).is_err() {
                    self.panic(EventError::QueueFull);
                }
            }
            _ => {}
        }
    }
}
#[cfg(test)]
mod tests {
    use core::fmt::{Display, Formatter};
    use std::fmt::Arguments;
    use std::panic::AssertUnwindSafe;
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        pub MotorEnabler {}
        impl MotorEnabler for MotorEnabler {
            fn set_enable(&mut self, enabled: bool);
        }
    }
    #[derive(Debug)]
    #[derive(Copy)]
#[derive(Clone)]
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
                TestError::Panic => {ErrorSeverity::Panic}
                TestError::ImmediateShutdown => {ErrorSeverity::ImmediateShutdown}
                TestError::GracefulShutdown => {ErrorSeverity::GracefulShutdown}
                TestError::Report => {ErrorSeverity::Report}
                TestError::Ignore => {ErrorSeverity::Ignore}
            }
        }
    }
    pub struct TestLogger {}
    impl Logger for TestLogger {
        fn trace(&self, args: Arguments<'_>) {}

        fn debug(&self, args: Arguments<'_>) {}

        fn info(&self, args: Arguments<'_>) {}

        fn warn(&self, args: Arguments<'_>) {}

        fn error(&self, args: Arguments<'_>) {
            println!("Error log: {}", args)
        }
    }

#[test]
fn test_error_handler_ignore() {
    static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
    let mut mock_motor_enabler = MockMotorEnabler::new();
    let mut mock_logger = TestLogger{};
    let mut test_error_handler = ErrorHandler::new(&mut mock_motor_enabler, &mut mock_logger, &EVENT_QUEUE);
    test_error_handler.handle_error(TestError::Ignore);
}
    #[test]
    fn test_error_handler_report() {
        static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
        let mut mock_motor_enabler = MockMotorEnabler::new();
        let mut mock_logger = TestLogger{};
        let mut test_error_handler = ErrorHandler::new(&mut mock_motor_enabler, &mut mock_logger, &EVENT_QUEUE);
        test_error_handler.handle_error(TestError::Report);
        assert!(EVENT_QUEUE.dequeue() == None);
    }
    #[test]
    fn test_error_handler_graceful_shutdown() {
        static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
        let mut mock_motor_enabler = MockMotorEnabler::new();
        let mut mock_logger = TestLogger{};
        let mut test_error_handler = ErrorHandler::new(&mut mock_motor_enabler, &mut mock_logger, &EVENT_QUEUE);
        test_error_handler.handle_error(TestError::GracefulShutdown);
        assert!(EVENT_QUEUE.dequeue() == Some(GlobEvent::ErrorWithGracefulShutdown));
        assert!(EVENT_QUEUE.dequeue() == None);
    }

    #[test]
    fn test_error_handler_immediate_shutdown() {
        static EVENT_QUEUE: Q8<GlobEvent> = Q8::new();
        let mut mock_motor_enabler = MockMotorEnabler::new();
        mock_motor_enabler.expect_set_enable()
            .with(eq(false))
            .times(1)
            .return_const(());

        let mut mock_logger = TestLogger{};
        let mut test_error_handler = ErrorHandler::new(&mut mock_motor_enabler, &mut mock_logger, &EVENT_QUEUE);
        test_error_handler.handle_error(TestError::ImmediateShutdown);
        assert!(EVENT_QUEUE.dequeue() == Some(GlobEvent::ErrorWithImmediateShutdown));
        assert!(EVENT_QUEUE.dequeue() == None);
    }
}
