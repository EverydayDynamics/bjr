
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
    use crate::stepper_state::STEP_BASE_FREQ;
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

    //use rtt_target::{rprintln, rtt_init_print};
    use rtt_log;
    use log;
    const SYSFREQ: u32 = 100_000_000;
    // Shared resources go here
    #[shared]
    struct Shared {
        steppers_state: [StepperState;NUM_ACTUATOR],

    }

    // Local resources go here
    #[local]
    struct Local {
        button: PC13<Input>,
        controller_task: ControllerTask<LinearStepperMotor<'static,LGA201S06_A_UECB_019, SilentStepStick>>,
        stepper_task: StepperCtrlrTask<'static, PA5<Output>,PA0<Output>>,
    }
    #[monotonic(binds = TIM3, default = true)]
    type MicrosecMono = MonoTimer64Us<pac::TIM3>;
    #[init(local = [
    qa: Queue<FrequencyDrift, 2> = Queue::new(),
    qb: Queue<FrequencyDrift, 2> = Queue::new(),
    qc: Queue<FrequencyDrift, 2> = Queue::new(),
    qd: Queue<FrequencyDrift, 2> = Queue::new(),
    ])]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        //rtt_init_print!();
        //rprintln!("Initializing BJR...");
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
        let mut button = gpioc.pc13.into_pull_up_input();
        // led
        let stepper_a_stp_pin = gpioa.pa5.into_push_pull_output();
        let stepper_a_dir_pin = gpioa.pa0.into_push_pull_output();

        let stepper_b_stp_pin = gpioa.pa6.into_push_pull_output();
        let stepper_b_dir_pin = gpioa.pa7.into_push_pull_output();


        let stepper_d_stp_pin = gpioa.pa10.into_push_pull_output();
        let stepper_d_dir_pin = gpioa.pa11.into_push_pull_output();

        let (mut stp_setp_a_prod, mut stp_setp_a_cons) = ctx.local.qa.split();
        let (mut stp_setp_b_prod, mut stp_setp_b_cons) = ctx.local.qb.split();
        let (mut stp_setp_c_prod, mut stp_setp_c_cons) = ctx.local.qc.split();
        let (mut stp_setp_d_prod, mut stp_setp_d_cons) = ctx.local.qd.split();
        let stepper_task = StepperCtrlrTask::new(stepper_a_stp_pin, stepper_a_dir_pin, Default::default(), stp_setp_a_cons);

        //Setup tasks
        let mut controller_task = ControllerTask::new([
            LinearStepperMotor::new(LGA201S06_A_UECB_019::new(), stp_setp_a_prod, SilentStepStick::new()),
            LinearStepperMotor::new(LGA201S06_A_UECB_019::new(), stp_setp_b_prod, SilentStepStick::new()),
            LinearStepperMotor::new(LGA201S06_A_UECB_019::new(), stp_setp_c_prod, SilentStepStick::new())]);
        let channels = (Channel1::new(gpioa.pa8),Channel2::new(gpioa.pa9));
        let pwm = ctx.device.TIM1.pwm_hz(channels, 15.kHz(), &clocks).split();
        let (mut ch1, _ch2) = pwm;
        let max_duty: u16 = ch1.get_max_duty();
        ch1.set_duty(max_duty/2);
        ch1.enable();
        // Setup timers
        let mut stepper_main_timer = ctx.device.TIM2.counter_hz(&clocks);
        stepper_main_timer.start(STEP_BASE_FREQ.Hz()).unwrap();
        stepper_main_timer.listen(Event::Update);

        let mono = ctx.device.TIM3.monotonic64_us(&clocks);
        stepper_task_runner::spawn().ok();
        controller_task_runner::spawn().ok();
        (
            Shared {
                // Initialization of shared resources go here
                steppers_state: Default::default()
            },
            Local {
                // Initialization of local resources go here
                button,
                controller_task,
                stepper_task,
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
    #[task(local = [controller_task], shared = [steppers_state], priority = 2)]
    fn controller_task_runner(mut ctx: controller_task_runner::Context) {
        ctx.shared.steppers_state.lock(|steppers_state| {
            ctx.local.controller_task.update_stepper_state(steppers_state);
        });
        ctx.local.controller_task.run();

        let a:systick_monotonic::fugit::Instant<u64, 1, 1000000>  = systick_monotonic::Systick::zero();
        controller_task_runner::spawn_at(a + ExtU64::millis(ctx.local.controller_task.next_run())).unwrap();
    }

    #[task(local = [stepper_task], shared = [steppers_state], priority = 3)]
    fn stepper_task_runner(mut ctx: stepper_task_runner::Context) {
        let mut micros  = monotonics::now().ticks();
        let pre_micros  = monotonics::now().ticks();
        ctx.shared.steppers_state.lock(|steppers_state| {

            let (stepper_state_ref, next_micros_ret) = ctx.local.stepper_task.run(micros);
            micros = next_micros_ret;
            steppers_state[0] = *stepper_state_ref;
        });

        let a:systick_monotonic::fugit::Instant<u64, 1, 1000000>  = systick_monotonic::Systick::zero();

        stepper_task_runner::spawn_at(a + ExtU64::micros(micros)).unwrap();
        let post_micros  = monotonics::now().ticks();
        log::debug!("t:{}",post_micros - pre_micros);
    }
    //#[task(binds = TIM2, shared = [stepper_controllers, steppers_state], local = [button])]
    //fn tim2(mut cx: tim2::Context) {

    //    // Safe access to local `static mut` variable
    //    (cx.shared.stepper_controllers, cx.shared.steppers_state).lock(|stepper_controllers, steppers_state| {
    //        steppers_state[0] = *stepper_controllers[0].run_first_stage();
    //        stepper_controllers[0].run_second_stage();
    //    }
    //    );
    //}
}
