use std::fmt::{Display, Formatter};
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
use bsp_traits::{Button, Logger, MotorEnabler, StepperMotorController};
use embedded_time::duration::Microseconds;
use bjr_bsp::boards::{BoardCreationError, BoardResources};
use bjr_logic::app::severity_trait::{ErrorSeverity, Severity};

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

pub fn build_application<BTN: Button, LOG: Logger ,ME: MotorEnabler, STPA:StepperMotorController, STPB:StepperMotorController, STPC:StepperMotorController>(current_usecs: u64, board: &mut dyn BoardResources<Button=BTN, LogDevice=LOG, MotorEnabler=ME, StepperDriveA=STPA, StepperDriveB=STPB, StepperDriveC=STPC>) -> LogicRunner {
    let event_queue = get_event_queue();
    let (mut motor_enabler, mut log_device) = board.get_infallible_resources();
    let mut error_handler = ErrorHandler::new(event_queue);
    match board.get_fallible_resources() {
        Ok((mut button, mut stp_a, mut stp_b, mut stp_c)) => {
            let button_handler = ButtonHandler::new(&mut button, get_event_queue());
            let steppers: [&mut dyn StepperMotorController;3]= [
                &mut stp_a,
                &mut stp_b,
                &mut stp_c,
            ];
            let motor_handler = MotorHandler::new(steppers, MotorPosLimit::new(true), MotorVelLimit::new(true));
            let io_manager = DefaultIOManager::new(motor_handler);
            let state_manager = StateManager::new(DefaultStateRunnerSelector::new(), io_manager);
            let event_handler = EventHandler::new(event_queue);
            LogicRunner::new(button_handler, event_queue, state_manager, event_handler, error_handler, &mut log_device, &mut motor_enabler, Microseconds(current_usecs))
        }
        Err(error) => {
            let error_binding = AppError::SetupError(error);
            error_handler.handle_error(&mut motor_enabler, &mut log_device, error_binding);
            panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
