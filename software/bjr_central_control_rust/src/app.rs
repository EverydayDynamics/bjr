
use rtic::app;
#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1,SPI2])]
mod app {
    use stm32f4xx_hal::{timer::{CounterUs, Event},
                        gpio::{gpioa::PA0,
                               gpioa::PA5,
                               gpioa::PA6,
                               gpioa::PA7,
                               gpioa::PA8,
                               gpioa::PA9,
                               gpioa::PA10,
                               gpioa::PA11,
                               gpioc::PC13, Input, Output}, prelude::*};
    use crate::stepper_state::StepperState;
    use crate::controller_task::ControllerTask;
    use crate::actuator_num::NUM_ACTUATOR;
    use crate::captive_linear_stepper::LGA201S06_A_UECB_019;
    use crate::motor::LinearStepperMotor;
    use crate::stepper_driver::SilentStepStick;
    use crate::stepper_controller2::StepperCtrlrTask;
    use systick_monotonic::{fugit::ExtU64, fugit::ExtU32, Systick};
    use heapless::spsc::Queue;
    use stm32f4xx_hal::pac;
    use stm32f4xx_hal::timer::MonoTimer64Us;
    use uom::si::f32::*;
    use stm32f4xx_hal::timer::Channel1;
    use stm32f4xx_hal::timer::Channel2;
    use core::sync::atomic::{AtomicI32, Ordering};

    //use rtt_target::{rprintln, rtt_init_print};
    use rtt_log;
    use log;
    const SYSFREQ: u32 = 150_000_000;
    static STEPPERS_STATE: [StepperState;NUM_ACTUATOR] = [StepperState::new(),StepperState::new(),StepperState::new()];
    // Shared resources go here
    #[shared]
    struct Shared {

    }

    // Local resources go here
    #[local]
    struct Local {
        controller_task: ControllerTask<LinearStepperMotor<'static,LGA201S06_A_UECB_019, SilentStepStick>>,
        stepper_task: StepperCtrlrTask<'static, PA5<Output>,PA0<Output>>,
        stepper_main_timer: CounterUs<stm32f4xx_hal::pac::TIM2>,
    }
    #[monotonic(binds = TIM3, default = true)]
    type MicrosecMono = MonoTimer64Us<pac::TIM3>;
    #[init(local = [
    qa: Queue<f32, 2> = Queue::new(),
    qb: Queue<f32, 2> = Queue::new(),
    qc: Queue<f32, 2> = Queue::new(),
    qd: Queue<f32, 2> = Queue::new(),
    ])]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_log::init();
        log::debug!("Application started");
        // syscfg
        let mut syscfg = ctx.device.SYSCFG.constrain();
        // clocks
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(SYSFREQ.Hz()).freeze();
        // gpio ports A and C
        let gpioa = ctx.device.GPIOA.split();
        let gpioc = ctx.device.GPIOC.split();
        // button
        // led
        let stepper_a_stp_pin = gpioa.pa5.into_push_pull_output();
        let stepper_a_dir_pin = gpioa.pa0.into_push_pull_output();

        let (mut stp_setp_a_prod, mut stp_setp_a_cons) = ctx.local.qa.split();
        let (mut stp_setp_b_prod, stp_setp_b_cons) = ctx.local.qb.split();
        let (mut stp_setp_c_prod, stp_setp_c_cons) = ctx.local.qc.split();

        let stepper_task = StepperCtrlrTask::new(stepper_a_stp_pin, stepper_a_dir_pin, Default::default(), stp_setp_a_cons);
        //Setup tasks
        let controller_task = ControllerTask::new([
            LinearStepperMotor::new(LGA201S06_A_UECB_019::new(), stp_setp_a_prod, SilentStepStick::new()),
            LinearStepperMotor::new(LGA201S06_A_UECB_019::new(), stp_setp_b_prod, SilentStepStick::new()),
            LinearStepperMotor::new(LGA201S06_A_UECB_019::new(), stp_setp_c_prod, SilentStepStick::new())]);

        // Setup timers
        let mut stepper_main_timer = ctx.device.TIM2.counter_us(&clocks);
        stepper_main_timer.start(ExtU32::micros(1000)).unwrap();
        stepper_main_timer.listen(Event::Update);

        let mono = ctx.device.TIM3.monotonic64_us(&clocks);
        controller_task_runner::spawn().ok();
        (
            Shared {
                // Initialization of shared resources go here
            },
            Local {
                // Initialization of local resources go here
                controller_task,
                stepper_task,
                stepper_main_timer,
            },
            init::Monotonics(mono),
        )
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }

    #[task(local = [controller_task], shared = [], priority = 2)]
    fn controller_task_runner(ctx: controller_task_runner::Context) {
            ctx.local.controller_task.update_stepper_state(&STEPPERS_STATE);
        ctx.local.controller_task.run();

        let a:systick_monotonic::fugit::Instant<u64, 1, 1000000>  = systick_monotonic::Systick::zero();
        controller_task_runner::spawn_at(a + ExtU64::millis(ctx.local.controller_task.next_run())).unwrap();
    }

    #[task(binds = TIM2, shared = [], local = [stepper_task, stepper_main_timer])]
    fn tim2(ctx: tim2::Context) {
        let mut micros  = monotonics::now().ticks();
        let pre_micros  = monotonics::now().ticks();
            micros = ctx.local.stepper_task.run(micros, &STEPPERS_STATE[0]);
        //let a:systick_monotonic::fugit::Instant<u64, 1, 1000000>  = systick_monotonic::Systick::zero();
        let post_micros  = monotonics::now().ticks();
        log::debug!("t:{}, w:{}",post_micros - pre_micros,micros);
        ctx.local.stepper_main_timer.start(ExtU32::micros(micros as u32)).unwrap();
    }
}
