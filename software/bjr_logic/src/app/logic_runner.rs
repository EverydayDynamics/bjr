use crate::app::button_handler::ButtonHandler;
use crate::app::error_handler::ErrorHandler;
use crate::app::event::GlobEvent;
use crate::app::event_handler::{EventHandler, EventHandlerError};
use crate::app::io_manager::DefaultIOManager;
use crate::app::menu_handler::MenuHandler;
use crate::app::parameter_manager::{parameter_manager, LogicRunnerPeriodUs};
use crate::app::severity_trait::{ErrorSeverity, Severity};
use crate::app::state_manager::StateManager;
use crate::app::state_runner::{StateRunnerCommand, StateRunnerContext};
use crate::app::state_runner_selector::DefaultStateRunnerSelector;
use device_traits::{Button, LoggableMessage, Logger, MotorEnabler, Reader, StepperMotorController, TelemetrySender, TouchSensor};
use core::fmt::{Display, Formatter, Write};
use embedded_time::duration::*;
use heapless::mpmc::Q8;
use crate::app::telemetry_handler::{TelemetryBuilder, TelemetryHandler, TelemetryHandlerError};
use crate::app::touch_handler::Differentiator;
use os_traits::TimeControl;

enum LogicRunnerError {
    TimeOverrun,
    EventHandlerError(EventHandlerError),
    TelementryError(TelemetryHandlerError),
}
impl LoggableMessage for LogicRunnerError {}
impl Display for LogicRunnerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            LogicRunnerError::TimeOverrun => {
                write!(f, "Time Overrun")
            }
            LogicRunnerError::EventHandlerError(error) => {
                write!(f, "Event handler error: {}", error)
            }
            LogicRunnerError::TelementryError(error) => {
                write!(f, "Telemetry sending error: {}", error)
            }
        }
    }
}
impl Severity for LogicRunnerError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            LogicRunnerError::TimeOverrun => ErrorSeverity::ImmediateShutdown,
            LogicRunnerError::EventHandlerError(_) => ErrorSeverity::Panic,
            LogicRunnerError::TelementryError(_) => ErrorSeverity::Report,
        }
    }
}
pub struct LogicRunner<'a, BTN, LOG, ME, MA, MB, MC, TS, MIO, TEL, DIFF, TIM>
where
    MA: StepperMotorController,
    MB: StepperMotorController,
    MC: StepperMotorController,
    TS: TouchSensor,
    MIO: Reader + Write,
    DIFF: Differentiator,
    TIM: TimeControl,
{
    next_call_time: Option<Microseconds<u64>>,
    last_call_time: Option<Microseconds<u64>>,
    last_end_time: Option<Microseconds<u64>>,
    button_handler: ButtonHandler<BTN>,
    event_queue: &'static Q8<GlobEvent>,
    event_handler: EventHandler,
    state_manager: StateManager<DefaultStateRunnerSelector>,
    io_manager: DefaultIOManager<MA, MB, MC, TS, DIFF>,
    error_handler: ErrorHandler,
    log_device: LOG,
    motor_enabler: ME,
    menu_handler: MenuHandler<'a, MIO>,
    state_runner_command: StateRunnerCommand,
    telemetry_handler: TelemetryHandler<TEL>,
    time_control: TIM
}

impl<'a, BTN, LOG, ME, MA, MB, MC, TS, MIO, TEL, DIFF, TIM> LogicRunner<'a, BTN, LOG, ME, MA, MB, MC, TS, MIO, TEL, DIFF, TIM>
where
    BTN: Button,
    LOG: Logger,
    ME: MotorEnabler,
    MA: StepperMotorController,
    MB: StepperMotorController,
    MC: StepperMotorController,
    TS: TouchSensor,
    MIO: Reader + Write,
    TEL: TelemetrySender,
    DIFF: Differentiator,
    TIM: TimeControl,
{
    pub fn new(
        button_handler: ButtonHandler<BTN>,
        event_queue: &'static Q8<GlobEvent>,
        state_manager: StateManager<DefaultStateRunnerSelector>,
        io_manager: DefaultIOManager<MA, MB, MC, TS, DIFF>,
        event_handler: EventHandler,
        error_handler: ErrorHandler,
        log_device: LOG,
        motor_enabler: ME,
        menu_handler: MenuHandler<'a, MIO>,
        telemetry_handler: TelemetryHandler<TEL>,
        time_control: TIM,
    ) -> Self {
        LogicRunner {
            next_call_time: None,
            last_end_time: None,
            last_call_time: None,
            button_handler,
            event_queue,
            event_handler,
            state_manager,
            io_manager,
            error_handler,
            log_device,
            motor_enabler,
            menu_handler,
            state_runner_command: StateRunnerCommand::NoCommand,
            telemetry_handler,
            time_control,
        }
    }
    fn handle_error<E>(&mut self, result: Result<(),E>)
    where E: Severity+LoggableMessage
    {
        if let Err(error) = result {
            self.error_handler.handle_error(
                &mut self.motor_enabler,
                &mut self.log_device,
                error,
            );
        }
    }
    fn send_cpu_use_telem(&mut self, period:u64, telem_builder:&mut TelemetryBuilder) {
        if let Some(last_end_time) = self.last_end_time {
            if let Some(last_call_time) = self.last_call_time {
                let call_duration = last_end_time - last_call_time;
                let cpu_use = call_duration.integer() as f32 /period as f32*100.0;
                telem_builder.add_cpu_use(cpu_use);
            }

        }
    }
    pub fn update(&mut self) -> Microseconds<u64> {
        let call_time = Microseconds::<u64>::new(self.time_control.get_tick());
        let period = parameter_manager().get::<LogicRunnerPeriodUs>() as u64;
        let next_call_time = if let Some(last_call_time) = self.next_call_time {
            last_call_time + Microseconds::<u64>::new(period)
        } else {
            call_time + Microseconds::<u64>::new(period)
        };
        let mut telemetry_builder = self.telemetry_handler.prepare_packet(call_time);
        if next_call_time < call_time {
            self.handle_error(Err(LogicRunnerError::TimeOverrun));
        } else {
            let button_handler_result = self.button_handler.update(call_time);
            self.handle_error(button_handler_result);
            match self.event_handler.handle_events(&mut self.log_device).map_err(LogicRunnerError::EventHandlerError)
            {
                Err(error) => {
                    self.handle_error(Err(error));
                }
                Ok((state, command)) => {
                    self.state_runner_command = command;
                    let mut state_ctx = StateRunnerContext{
                        iomanager: &mut self.io_manager,
                        call_time,
                        event_queue: self.event_queue,
                        logger: &mut self.log_device,
                        command: &self.state_runner_command,
                        telemetry_builder: &mut telemetry_builder,
                        motor_enabler: &mut self.motor_enabler,
                    };
                    let state_manager_result = self.state_manager.update(
                        state,
                        &mut state_ctx
                    );
                    self.handle_error(state_manager_result);
                }
            }
            let menu_result = self.menu_handler.update(&mut self.state_runner_command);
            self.handle_error(menu_result);
        }
        self.send_cpu_use_telem(period, &mut telemetry_builder);
            let telem_result = self.telemetry_handler.send_packet(telemetry_builder
                    .get_packet())
                .map_err(LogicRunnerError::TelementryError);
        self.handle_error(telem_result);
        self.next_call_time = Some(next_call_time);
        let end_time = self.time_control.get_tick();
        self.last_end_time = Some(Microseconds::<u64>::new(end_time));
        self.last_call_time = Some(call_time);
        next_call_time
    }
}
