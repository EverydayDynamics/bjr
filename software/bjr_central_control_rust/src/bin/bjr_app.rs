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
peripherals = false,
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
        pac::TIM2,
    };
    use core::cell::RefCell;
    use core::default;
    use core::fmt::{Display, Formatter};
    use bjr_bsp;
    use bjr_bsp::Board;
    use bjr_bsp::boards::{BoardResources, BoardCreationError};
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
    use bjr_logic::app::severity_trait::{ErrorSeverity, Severity};
    use bjr_logic::app::state_manager::StateManager;
    use bjr_logic::app::state_runner_selector::DefaultStateRunnerSelector;
    use bsp_traits::{MotorInput, StepperMotorController};
    use cortex_m::interrupt::{self, Mutex};
    use defmt::error;
    use rtic_monotonics;
    use rtic_monotonics::stm32::*;
    use rtic_monotonics::stm32::Tim2 as Mono;
    use rtic_monotonics::Monotonic;
    use embedded_time::duration::*;
    use rtic_monotonics::stm32::fugit::Instant;
    use stm32f4xx_hal::pac::Peripherals;
    use stm32f4xx_hal::rcc::Clocks;
    pub enum AppError {
        SetupError(BoardCreationError)
    }
    impl Severity for AppError {fn get_severity(&self) -> ErrorSeverity {
        match self{
            AppError::SetupError(_) => { ErrorSeverity::Panic }
        }
    }

    }
    impl Display for AppError {fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self{
            AppError::SetupError(bce) => { write!(f,"ApplicationError, Board initialization failed: {}", bce) }
        }
    }

    }
    // Shared resources go here
    #[shared]
    struct Shared {
        // TODO: Add resources
    }

    // Local resources go here
    #[local]
    struct Local {
    }
    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {

        defmt::error!("Init start");
        defmt::error!("Init finished");
        task1::spawn().unwrap();
        (
                    Shared {
                        // Initialization of shared resources go here
                    },
                    Local {
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
        let mut board = Board::new();
        let token = rtic_monotonics::create_stm32_tim2_monotonic_token!();
        let timer_clock_hz = 75_000_000; // ??????????????????????????????
        // Start the monotonic
        Mono::start(timer_clock_hz, token);
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
                let mut logic_runner = LogicRunner::new(button_handler, event_queue, state_manager, event_handler, error_handler, &mut log_device, &mut motor_enabler, Microseconds(Mono::now().ticks()));
                loop {
                    let now = Mono::now().ticks();
                    let next_run = logic_runner.update(embedded_time::duration::Microseconds(now));
                    let baba: Instant<u64, 1, 1000000> = Instant::<u64, 1, 1000000>::from_ticks(next_run.integer());
                    Mono::delay_until(baba).await;
                }
            }
            Err(error) => {
                let error_binding = AppError::SetupError(error);
                error_handler.handle_error(&mut motor_enabler, &mut log_device, error_binding);
            }
        }
        {

        }
        task1::spawn().ok();
    }
}