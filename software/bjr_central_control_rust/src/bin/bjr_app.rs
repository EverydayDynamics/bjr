#![no_main]
#![no_std]

use bjr as _; // global logger + panicking-behavior + memory layout

// TODO(7) Configure the `rtic::app` macro

#[rtic::app(
// TODO: Replace `some_hal::pac` with the path to the PAC
device = stm32f4xx_hal::pac,
peripherals = false,
// TODO: Replace the `FreeInterrupt1, ...` with free interrupt vectors if software tasks are used
// You can usually find the names of the interrupt vectors in the some_hal::pac::interrupt enum.
dispatchers = [SPI2]
)]
mod app {
    use bjr_bsp;
    use bjr_bsp::boards::{BoardCreationError, BoardResources};
    use bjr_bsp::Board;
    use bjr_builder::build_application;
    use bjr_logic::app::menu_handler::MenuContext;
    use bjr_logic::app::severity_trait::{ErrorSeverity, Severity};
    use bsp_traits::StepperMotorController;
    use core::fmt::{Display, Formatter, Write};
    use embedded_time::duration::*;
    use rtic_monotonics;
    use rtic_monotonics::stm32::fugit::Instant;
    use rtic_monotonics::stm32::Tim2 as Mono;
    use rtic_monotonics::stm32::*;
    use rtic_monotonics::Monotonic;
    use stm32f4xx_hal::prelude::*;

    pub enum AppError {
        SetupError(BoardCreationError),
    }
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
    // Shared resources go here
    #[shared]
    struct Shared {
        // TODO: Add resources
    }

    // Local resources go here
    #[local]
    struct Local {}
    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        task1::spawn().unwrap();
        (
            Shared {
                        // Initialization of shared resources go here
                    },
            Local {},
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
        let mut menu_context = MenuContext::default();

        let mut logic_runner = build_application(&mut board, &mut menu_context);
        loop {
            let now = Mono::now().ticks();
            let next_run = logic_runner.update(Microseconds(now));
            let baba: Instant<u64, 1, 1000000> =
                Instant::<u64, 1, 1000000>::from_ticks(next_run.integer());
            Mono::delay_until(baba).await;
        }
    }
}
