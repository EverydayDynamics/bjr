#![no_std]
use bjr_logic::app::button_handler::ButtonHandler;
use bjr_logic::app::error_handler::ErrorHandler;
use bjr_logic::app::event_handler::EventHandler;
use bjr_logic::app::event_queue::get_event_queue;
use bjr_logic::app::io_manager::DefaultIOManager;
use bjr_logic::app::limits::{MotorPosLimit, MotorVelLimit};
use bjr_logic::app::logic_runner::LogicRunner;
use bjr_logic::app::motor_handler::MotorHandler;
use bjr_logic::app::state_manager::StateManager;
use bjr_logic::app::state_runner_selector::DefaultStateRunnerSelector;
use bsp_traits::{Button, Logger, MotorEnabler, Reader, StepperMotorController, TouchSensor};
use bjr_bsp::boards::{BoardCreationError, BoardResources};
use bjr_logic::app::severity_trait::{ErrorSeverity, Severity};
use core::fmt::{Display, Formatter, Write};
use bjr_logic::app::menu_handler::{MenuContext, MenuHandler};
use bjr_logic::app::touch_handler::TouchHandler;
use embedded_time::duration::Microseconds;

pub enum AppError {
    SetupError(BoardCreationError)
}
impl Severity for AppError {fn get_severity(&self) -> ErrorSeverity {
    match self{
        AppError::SetupError(_) => { ErrorSeverity::Panic }
    }
}

}
impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            AppError::SetupError(bce) => { write!(f, "ApplicationError, Board initialization failed: {}", bce) }
        }
    }
}

pub fn build_application<'a, BTN: Button, LOG: Logger ,ME: MotorEnabler, STPA:StepperMotorController, STPB:StepperMotorController, STPC:StepperMotorController, TS:TouchSensor,MIO: Reader+Write>(board: &mut dyn BoardResources<Button=BTN, LogDevice=LOG, MotorEnabler=ME, StepperDriveA=STPA, StepperDriveB=STPB, StepperDriveC=STPC, TouchSensor=TS, MenuIO=MIO>, menu_context: &'a mut MenuContext) -> LogicRunner<'a, BTN, LOG, ME, STPA, STPB, STPC, TS, MIO> {
    let event_queue = get_event_queue();
    let (mut motor_enabler, mut log_device, mut menu_io) = board.get_infallible_resources();
    let mut error_handler = ErrorHandler::new(event_queue);
    match board.get_fallible_resources() {
        Ok((mut button, mut stp_a, mut stp_b, mut stp_c, mut touch_sensor)) => {
            let menu_handler = MenuHandler::new(menu_io, menu_context);
            let button_handler = ButtonHandler::new(button, get_event_queue());
            let motor_handler = MotorHandler::new(stp_a, stp_b, stp_c, MotorPosLimit::new(true), MotorVelLimit::new(true));
            let touch_handler = TouchHandler::new(touch_sensor, Microseconds::default(), None);
            let io_manager = DefaultIOManager::new(motor_handler, touch_handler);
            let state_manager = StateManager::new(DefaultStateRunnerSelector::new(), io_manager);
            let event_handler = EventHandler::new(event_queue);
            LogicRunner::new(button_handler, event_queue, state_manager, event_handler, error_handler, log_device, motor_enabler, menu_handler)
        }
        Err(error) => {
            let error_binding = AppError::SetupError(error);
            error_handler.handle_error(&mut motor_enabler, &mut log_device, error_binding);
            panic!()
        }
    }
}

