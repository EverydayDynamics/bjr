#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use stm32f4xx_hal::{timer::{CounterUs, Event, Timer3},
                        gpio::{gpioa::PA3, gpioa::PA2,gpioa::PA5, gpioc::PC0, Edge, Input, Output, PushPull}, hal, prelude::*};
    use stm32f4xx_hal::pac::Interrupt;
    use crate::stepper_control::StepperCtrlr;
    use crate::stepper_control::StepperCtrlTrait;
    const SYSFREQ: u32 = 100_000_000;
    // Shared resources go here
    #[shared]
    struct Shared {
        stepper_second_timer: CounterUs<stm32f4xx_hal::pac::TIM3>,
        stepperctrl: StepperCtrlr<PA5<Output>,PA2<Output>>

    }

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
        let clocks = rcc.cfgr.sysclk(SYSFREQ.Hz()).use_hse(100.MHz()).freeze();
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
        let stepper1_dir_pin = gpioa.pa2.into_push_pull_output();
        let stepperctrl = StepperCtrlr::new(stepper1_stp_pin, stepper1_dir_pin, 0,0);

        // Setup timers
        let mut stepper_main_timer = ctx.device.TIM2.counter_hz(&clocks);
        let mut stepper_second_timer = ctx.device.TIM3.counter_us(&clocks);
        stepper_main_timer.start(10.kHz()).unwrap();
        stepper_main_timer.listen(Event::Update);

        stepper_second_timer.listen(Event::Update);


        (
            Shared {
                // Initialization of shared resources go here
                stepper_second_timer,
                stepperctrl,
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
    #[task(binds = TIM2, shared = [stepperctrl, stepper_second_timer])]
    fn tim2(mut cx: tim2::Context) {
        // Safe access to local `static mut` variable
        cx.shared.stepperctrl.lock(|stepperctrl| {
            stepperctrl.run_first_stage(Some(1));

        }
        );
        cx.shared.stepper_second_timer.lock(|stepper_second_timer| {
            stepper_second_timer.start(10.micros()).unwrap();

        }
        );
    }
    #[task(binds = TIM3, shared = [stepperctrl, stepper_second_timer])]
    fn tim3(mut cx: tim3::Context) {
        // Safe access to local `static mut` variable
        cx.shared.stepper_second_timer.lock(|stepper_second_timer| {
            stepper_second_timer.cancel().unwrap();

        }
        );
        cx.shared.stepperctrl.lock(|stepperctrl| {
            stepperctrl.run_second_stage();

        }
        );
    }
}
