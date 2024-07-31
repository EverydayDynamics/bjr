#![no_main]
#![no_std]

use bjr_bsp::boards::{BjrBoardSupport, BoardCreationError};
use bjr_bsp::main_board::MyBoard;
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
use bjr as _;
#[cortex_m_rt::entry]
fn main() -> ! {
    match MyBoard::new() {
        Ok(mut board) => {
            let event_queue = get_event_queue();
            let mut board_resources = board.get_resources();
            let button_handler = ButtonHandler::new(board_resources.button.take().unwrap(),event_queue);
            let motor_handler = MotorHandler::new(board_resources.stepper_devicees.take().unwrap(), MotorPosLimit::new(true), MotorVelLimit::new(true));
            let io_manager = DefaultIOManager::new(motor_handler);
            let state_manager = StateManager::new(DefaultStateRunnerSelector::new(), io_manager);
            let event_handler = EventHandler::new();
            let error_handler = ErrorHandler::new(board_resources.motor_enabler.take().unwrap(),board_resources.log_device.take().unwrap(), event_queue);
            let logic_runner = LogicRunner::new(button_handler, event_queue, state_manager, event_handler, error_handler);
            defmt::println!("BSP loaded!");
        }
        Err(error) => {
            defmt::error!("Failed to load BSP: {}", defmt::Display2Format(&error));
        }
    }
    bjr::exit();
}

