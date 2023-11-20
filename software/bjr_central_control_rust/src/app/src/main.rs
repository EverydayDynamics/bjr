#![deny(unsafe_code)]
//#![deny(warnings)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
use panic_halt as _;

mod stepper_control;

#[cfg(not(test))]
#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use stm32f4xx_hal::{gpio::{gpioa::PA5, gpioc::PC0, Edge, Input, Output, PushPull}, hal, prelude::*};
    use crate::stepper_control::StepperCtrlr;
    use stm32f4xx_hal::{
        prelude::*,
        timer::{Event, Timer},
    };
    const SYSFREQ: u32 = 100_000_000;
    // Shared resources go here
    #[shared]
    struct Shared {}

    // Local resources go here
    #[local]
    struct Local {
        button: PC0<Input>,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        // syscfg
        let mut syscfg = ctx.device.SYSCFG.constrain();
        // clocks
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(SYSFREQ.Hz()).use_hse(25.MHz()).freeze();
        // gpio ports A and C
        let gpioa = ctx.device.GPIOA.split();
        let gpioc = ctx.device.GPIOC.split();
        // button
        let mut button = gpioc.pc0.into_pull_up_input();
        button.make_interrupt_source(&mut syscfg);
        button.enable_interrupt(&mut ctx.device.EXTI);
        button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Falling);
        // led
        let stepper1_stp_pin = gpioa.pa5.into_push_pull_output();
        let stepper1_dir_pin = gpioa.pa3.into_push_pull_output();
        let _stepperctrl = StepperCtrlr::new(stepper1_stp_pin, stepper1_dir_pin, 0,0);
        (
            Shared {
                // Initialization of shared resources go here
            },
            Local {
                // Initialization of local resources go here
                button,
            },
            init::Monotonics(),
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }
}

