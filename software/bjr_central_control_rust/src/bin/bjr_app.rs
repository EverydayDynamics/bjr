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
dispatchers = [SPI1]
)]
mod app {
    use stm32f4xx_hal::{
        prelude::*,
    };
    use rtic_monotonics::stm32::fugit::Instant;
    use core::fmt::{Display, Formatter};
    use bjr_bsp;
    use bjr_bsp::Board;
    use bjr_bsp::boards::{BoardCreationError, BoardResources};
    use bjr_builder::build_application;
    use bjr_logic::app::severity_trait::{ErrorSeverity, Severity};
    use bsp_traits::StepperMotorController;
    use rtic_monotonics;
    use rtic_monotonics::stm32::*;
    use rtic_monotonics::Monotonic;
    use embedded_time::duration::*;
    use rtic_monotonics::stm32::Tim2 as Mono;


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
        let now = Mono::now().ticks();
        let baba: Instant<u64, 1, 1000000> = Instant::<u64, 1, 1000000>::from_ticks(1000000);
        Mono::delay_until(baba).await;
        let mut logic_runner = build_application(now, &mut board);
            loop {
                let now = Mono::now().ticks();
                let next_run = logic_runner.update(Microseconds(now));
                let baba: Instant<u64, 1, 1000000> = Instant::<u64, 1, 1000000>::from_ticks(next_run.integer());
                Mono::delay_until(baba).await;
            }
    }
}