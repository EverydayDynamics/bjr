#![no_main]
#![no_std]

use bjr as _; // global logger + panicking-behavior + memory layout
use bjr_logic::app::logic_runner::LogicRunner;
use bjr_logic::app::button_handler::ButtonHandler;
use bjr_bsp::main_board::MyBoard;
// TODO(7) Configure the `rtic::app` macro

#[rtic::app(
// TODO: Replace `some_hal::pac` with the path to the PAC
device = stm32f4xx_hal::pac,
// TODO: Replace the `FreeInterrupt1, ...` with free interrupt vectors if software tasks are used
// You can usually find the names of the interrupt vectors in the some_hal::pac::interrupt enum.
dispatchers = [SPI1]
)]
mod app {

    use rtic_monotonics::systick::*;
    use stm32f4xx_hal::{
        gpio::{Output, PC13},
        pac,
        prelude::*,
    };
    use core::cell::RefCell;
    use bjr_bsp::boards::BjrBoardSupport;
    use bjr_bsp::main_board::MyBoard;
    use bjr_logic::app::button_handler::ButtonHandler;
    use bjr_logic::app::error_handler::ErrorHandler;
    use bjr_logic::app::event_handler::EventHandler;
    use bjr_logic::app::event_queue::get_event_queue;
    use bjr_logic::app::io_manager::DefaultIOManager;
    use bjr_logic::app::limits::{MotorPosLimit, MotorVelLimit};
    use bjr_logic::app::logic_runner;
    use bjr_logic::app::logic_runner::LogicRunner;
    use bjr_logic::app::motor_handler::MotorHandler;
    use bjr_logic::app::state_manager::StateManager;
    use bjr_logic::app::state_runner_selector::DefaultStateRunnerSelector;
    use bsp_traits::{MotorInput, StepperMotorController};
    use cortex_m::interrupt::{self, Mutex};
    use defmt::error;

    // Shared resources go here
    #[shared]
    struct Shared {
        // TODO: Add resources
    }

    // Local resources go here
    #[local]
    struct Local {
        // TODO: Add resources
    }
    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board_result = MyBoard::new();
        match board_result {
            Ok(mut board) => {
                let event_queue = get_event_queue();
                let mut board_resources = board.get_resources();
                let button_handler = ButtonHandler::new(board_resources.button.take().unwrap(),event_queue);
                let motor_handler = MotorHandler::new(board_resources.stepper_devices.take().unwrap(), MotorPosLimit::new(true), MotorVelLimit::new(true));
                let io_manager = DefaultIOManager::new(motor_handler);
                let state_manager = StateManager::new(DefaultStateRunnerSelector::new(), io_manager);
                let event_handler = EventHandler::new();
                let error_handler = ErrorHandler::new(board_resources.motor_enabler.take().unwrap(),board_resources.log_device.take().unwrap(), event_queue);
                let logic_runner = LogicRunner::new(button_handler, event_queue, state_manager, event_handler, error_handler);
            }
            Err(error) => {
                defmt::error!("Couldn't create board: {}", defmt::Display2Format(&error));
                panic!();
            }
        }
        //let logic_runner = LogicRunner::new((), &Default::default(), (), (), ());
        // TODO setup monotonic if used
        let rcc = cx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(150.MHz()).freeze();
        let a  = clocks.sysclk();

        let sysclk = { a.to_Hz()/* clock setup + returning sysclk as an u32 */ };
        let token = rtic_monotonics::create_systick_token!();
        rtic_monotonics::systick::Systick::start(cx.core.SYST, sysclk, token);
        task1::spawn().ok();
        (
            Shared {
                // Initialization of shared resources go here
            },
            Local {
                // Initialization of local resources go here
            },
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {

        loop {
            continue;
        }
    }

    // TODO: Add tasks
    #[task(priority = 1)]
    async fn task1(cx: task1::Context) {
        bjr::exit()
    }
}