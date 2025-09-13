#![no_std]
use board_support_package::boards::{BoardCreationError, BoardResources};
use application_logic::app::button_handler::ButtonHandler;
use application_logic::app::error_handler::ErrorHandler;
use application_logic::app::event_handler::EventHandler;
use application_logic::app::event_queue::get_event_queue;
use application_logic::app::io_manager::DefaultIOManager;
use application_logic::app::limits::{LimitLevel, MotorPosLimit, MotorVelLimit};
use application_logic::app::logic_runner::LogicRunner;
use application_logic::app::menu_handler::{MenuContext, MenuHandler};
use application_logic::app::motor_handler::MotorHandler;
use application_logic::app::severity_trait::{ErrorSeverity, Severity};
use application_logic::app::state_manager::StateManager;
use application_logic::app::state_runner_selector::DefaultStateRunnerSelector;
use application_logic::app::telemetry_handler::TelemetryHandler;
use application_logic::app::touch_handler::{SGDifferentiator, TouchHandler};
use core::fmt::{Display, Formatter, Write};
use device_traits::{
    Button, LoggableMessage, Logger, MotorEnabler, Reader, StepperMotorController, TelemetrySender,
    TouchSensor,
};
use os_traits::TimeControl;

pub enum AppError {
    SetupError(BoardCreationError),
}
impl LoggableMessage for AppError {}
impl Severity for AppError {
    fn get_severity(&self) -> ErrorSeverity {
        match self {
            AppError::SetupError(_) => ErrorSeverity::Panic,
        }
    }
}
impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AppError::SetupError(bce) => {
                write!(f, "ApplicationError, Board initialization failed: {}", bce)
            }
        }
    }
}
pub fn build_application<
    'a,
    BTN: Button,
    LOG: Logger,
    ME: MotorEnabler,
    STPA: StepperMotorController,
    STPB: StepperMotorController,
    STPC: StepperMotorController,
    TS: TouchSensor,
    MIO: Reader + Write,
    TEL: TelemetrySender,
    TIM: TimeControl,
>(
    board: &mut dyn BoardResources<
        Button = BTN,
        LogDevice = LOG,
        MotorEnabler = ME,
        StepperDriveA = STPA,
        StepperDriveB = STPB,
        StepperDriveC = STPC,
        TouchSensor = TS,
        MenuIO = MIO,
        TelemetrySender = TEL,
    >,
    menu_context: &'a mut MenuContext,
    time_control: TIM,
) -> LogicRunner<'a, BTN, LOG, ME, STPA, STPB, STPC, TS, MIO, TEL, SGDifferentiator<5, 3>, TIM> {
    let event_queue = get_event_queue();
    let (mut motor_enabler, mut log_device, menu_io) = board.get_infallible_resources();
    let mut error_handler = ErrorHandler::new(event_queue);
    match board.get_fallible_resources() {
        Ok((button, stp_a, stp_b, stp_c, touch_sensor, telemetry)) => {
            let motor_handler = MotorHandler::new(
                stp_a,
                stp_b,
                stp_c,
                MotorPosLimit::new(LimitLevel::Error),
                MotorVelLimit::new(LimitLevel::Error),
            );
            let touch_handler = TouchHandler::new(
                touch_sensor,
                [SGDifferentiator::new(), SGDifferentiator::new()],
            );
            LogicRunner::new(
                ButtonHandler::new(button, get_event_queue()),
                event_queue,
                StateManager::new(DefaultStateRunnerSelector::new()),
                DefaultIOManager::new(motor_handler, touch_handler),
                EventHandler::new(event_queue),
                error_handler,
                log_device,
                motor_enabler,
                MenuHandler::new(menu_io, menu_context),
                TelemetryHandler::new(telemetry),
                time_control,
            )
        }
        Err(error) => {
            let error_binding = AppError::SetupError(error);
            error_handler.handle_error(&mut motor_enabler, &mut log_device, error_binding);
            panic!()
        }
    }
}
