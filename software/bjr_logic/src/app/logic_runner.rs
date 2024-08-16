use core::fmt::{Display, Formatter, Write};
use bsp_traits::{Button, Logger, MotorEnabler, Reader, StepperMotorController, TouchSensor};
use heapless::mpmc::Q8;
use crate::app::event::GlobEvent;
use crate::app::button_handler::ButtonHandler;
use crate::app::event_handler::{EventHandler, EventHandlerError};
use embedded_time::duration::*;
use crate::app::error_handler::ErrorHandler;
use crate::app::io_manager::{DefaultIOManager, IOManager};
use crate::app::menu_handler::MenuHandler;
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
pub struct LogicRunner<'a, BTN, LOG, ME, MA, MB, MC, TS, MIO>
    where
        MA: StepperMotorController,
        MB: StepperMotorController,
        MC: StepperMotorController,
        TS: TouchSensor,
        MIO: Reader+Write,
{
    next_call_time: Option<Microseconds<u64>>,
    button_handler: ButtonHandler<BTN>,
    event_queue: &'static Q8<GlobEvent>,
    event_handler: EventHandler,
    state_manager: StateManager<DefaultStateRunnerSelector, DefaultIOManager<MA, MB, MC, TS>>,
    error_handler: ErrorHandler,
    log_device: LOG,
    motor_enabler: ME,
    menu_handler: MenuHandler<'a, MIO>
}

impl<'a, BTN, LOG, ME, MA, MB, MC, TS, MIO> LogicRunner<'a, BTN, LOG, ME, MA, MB, MC, TS, MIO>
where
BTN: Button,
LOG: Logger,
ME: MotorEnabler,
MA: StepperMotorController,
MB: StepperMotorController,
MC: StepperMotorController,
TS: TouchSensor,
MIO: Reader+Write,
{
    pub fn new(
        button_handler: ButtonHandler<BTN>,
        event_queue: &'static Q8<GlobEvent>,
        state_manager: StateManager<DefaultStateRunnerSelector, DefaultIOManager<MA, MB, MC, TS>>,
        event_handler: EventHandler,
        error_handler: ErrorHandler,
        log_device: LOG,
        motor_enabler: ME,
        menu_handler: MenuHandler<'a, MIO>
        ) -> Self {
        LogicRunner{
            next_call_time: None,
            button_handler,
            event_queue,
            event_handler,
            state_manager,
            error_handler,
            log_device,
            motor_enabler,
            menu_handler,
        }
    }
    pub fn update(&mut self, call_time: Microseconds<u64>) -> Microseconds<u64>{
        let period = parameter_manager().get::<LogicRunnerPeriodUs>() as u64;
        let next_call_time =if let Some(mut last_call_time) = self.next_call_time {
            last_call_time + Microseconds::<u64>::new(period)
        } else {
            call_time + Microseconds::<u64>::new(period)
        };
        if next_call_time < call_time {
            self.error_handler.handle_error(&mut self.motor_enabler, &mut self.log_device, LogicRunnerError::TimeOverrun);
        } else {
            if let Err(button_handler_error) = self.button_handler.update(call_time) {
                self.error_handler.handle_error(&mut self.motor_enabler, &mut self.log_device, button_handler_error);
            }
            match self.event_handler.handle_events(&mut self.log_device).map_err(|e| LogicRunnerError::EventHandlerError(e)) {
                Err(error) => {
                    self.error_handler.handle_error(&mut self.motor_enabler, &mut self.log_device, error);
                }
                Ok(state) => {
                    let state_runner_result = self.state_manager.update(state, call_time, self.event_queue, &mut self.motor_enabler, &mut self.log_device);
                    if let Err(state_runner_error) = state_runner_result {
                        self.error_handler.handle_error(&mut self.motor_enabler, &mut self.log_device, state_runner_error);
                    }
                }
            }
            if let Err(error) = self.menu_handler.update() {
                self.error_handler.handle_error(&mut self.motor_enabler, &mut self.log_device, error);
            }
        }
        self.next_call_time = Some(next_call_time);
        next_call_time
    }
}