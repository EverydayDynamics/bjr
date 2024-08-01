use core::fmt::{Display, Formatter};
use bsp_traits::{Logger, MotorEnabler};
use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;
use crate::app::button_handler::ButtonHandler;
use crate::app::event_handler::{EventHandler, EventHandlerError};
use embedded_time::duration::*;
use crate::app::error_handler::ErrorHandler;
use crate::app::io_manager::{DefaultIOManager, IOManager};
use crate::app::parameter_manager::{LogicRunnerPeriodUs, parameter_manager};
use crate::app::severity_trait::{ErrorSeverity, Severity};
use crate::app::state_manager::StateManager;
use crate::app::state_runner_selector::StateRunnerSelector;
use crate::app::state_runner_selector::DefaultStateRunnerSelector;
use crate::str_to_display;
use crate::utils::DisplayStr;

#[derive(Copy, Clone)]
enum LogicRunnerError {
    TimeOverrun,
    EventHandlerError(EventHandlerError),
}
impl Display for LogicRunnerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            LogicRunnerError::TimeOverrun => {write!(f, "Time Overrun")}
            LogicRunnerError::EventHandlerError(error) => {write!(f, "Event handler error: {}",error)}
        }

    }
}
impl Severity for LogicRunnerError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            LogicRunnerError::TimeOverrun => {ErrorSeverity::ImmediateShutdown},
            &LogicRunnerError::EventHandlerError(_) => {ErrorSeverity::Panic}
        }
    }
}
pub struct LogicRunner<'a> {
    next_call_time: Microseconds<u64>,
    button_handler: ButtonHandler<'a>,
    event_queue: &'static Q8<GlobEvent>,
    event_handler: EventHandler,
    state_manager: StateManager<DefaultStateRunnerSelector, DefaultIOManager<'a>>,
    error_handler: ErrorHandler,
    log_device: &'a mut dyn Logger,
    motor_enabler: &'a mut dyn MotorEnabler,
}

impl<'a> LogicRunner<'a>
{
    pub fn new(
        button_handler: ButtonHandler<'a>,
        event_queue: &'static Q8<GlobEvent>,
        state_manager: StateManager<DefaultStateRunnerSelector, DefaultIOManager<'a>>,
        event_handler: EventHandler,
        error_handler: ErrorHandler,
        log_device: &'a mut dyn Logger,
        motor_enabler: &'a mut dyn MotorEnabler,
        call_time: Microseconds<u64>,
        ) -> Self {
        LogicRunner{
            next_call_time: call_time,
            button_handler,
            event_queue,
            event_handler,
            state_manager,
            error_handler,
            log_device,
            motor_enabler,
        }
    }
    pub fn update(&mut self, call_time: Microseconds<u64>) -> Microseconds<u64>{
        let period = parameter_manager().get::<LogicRunnerPeriodUs>() as u64;
        self.next_call_time = self.next_call_time + Microseconds::<u64>::new(period);
        if self.next_call_time < call_time {
            self.error_handler.handle_error(self.motor_enabler, self.log_device, LogicRunnerError::TimeOverrun);
        } else {
            if let Err(button_handler_error) = self.button_handler.update(call_time) {
                self.error_handler.handle_error(self.motor_enabler, self.log_device, button_handler_error);
            }
            match self.event_handler.handle_events(self.log_device).map_err(|e|LogicRunnerError::EventHandlerError(e)) {
                Err(error) => {
                    self.error_handler.handle_error(self.motor_enabler, self.log_device, error);
                }
                Ok(state) => {
                    let state_runner_result = self.state_manager.update(state, call_time, self.event_queue, self.motor_enabler);
                    if let Err(state_runner_error) = state_runner_result {
                        self.error_handler.handle_error(self.motor_enabler, self.log_device, state_runner_error);
                    }
                }
            }
        }
        self.next_call_time
    }
}