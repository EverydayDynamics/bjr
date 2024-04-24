#![no_main]
#![no_std]
//#![feature(type_alias_impl_trait)]

use bjr as _; // global logger + panicking-behavior + memory layout

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
    use cortex_m::interrupt::{self, Mutex};
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