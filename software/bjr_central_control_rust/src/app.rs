
use rtic::app;
#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use stm32f4xx_hal::{timer::{CounterUs, Event, Timer3},
                        gpio::{gpioa::PA3,
                               gpioa::PA0,
                               gpioa::PA5,
                               gpioa::PA6,
                               gpioa::PA7,
                               gpioa::PA8,
                               gpioa::PA9,
                               gpioc::PC13, Edge, Input, Output, PushPull}, hal, prelude::*};
    use stm32f4xx_hal::pac::Interrupt;
    use crate::stepper_control::StepperCtrlr;
    use crate::stepper_control::StepperCtrlTrait;
    use crate::stepper_state::STEP_BASE_FREQ;
    use crate::stepper_state::StepperState;
    use crate::stepper_governor::{StackableStepperCtrlr};
    use crate::controller_task::ControllerTask;
    use crate::actuator_num::NUM_ACTUATOR;
    use crate::captive_linear_stepper::LGA201S06_A_UECB_019;
    use crate::motor::LinearStepperMotor;
    use systick_monotonic::{fugit::ExtU64, fugit::ExtU32, Systick};
    use heapless::spsc::Queue;
    //use rtt_target::{rprintln, rtt_init_print};
    use rtt_log;
    use log;
    const SYSFREQ: u32 = 100_000_000;
    // Shared resources go here
    #[shared]
    struct Shared {
        stepper_second_timer: CounterUs<stm32f4xx_hal::pac::TIM3>,
        stepper_controllers: [StackableStepperCtrlr<
            StepperCtrlr<'static, PA5<Output>,PA0<Output>>,
            StepperCtrlr<'static, PA6<Output>,PA7<Output>>,
            StepperCtrlr<'static, PA8<Output>,PA9<Output>>>; NUM_ACTUATOR],
        steppers_state: [StepperState;NUM_ACTUATOR],

    }

    // Local resources go here
    #[local]
    struct Local {
        button: PC13<Input>,
        controller_task: ControllerTask<LinearStepperMotor<'static,LGA201S06_A_UECB_019>>,
    }
    #[monotonic(binds = SysTick, default = true)]
    type Tonic = Systick<1000>;
    #[init(local = [
    qa: Queue<i32, 2> = Queue::new(),
    qb: Queue<i32, 2> = Queue::new(),
    qc: Queue<i32, 2> = Queue::new(),
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

        let stepper_c_stp_pin = gpioa.pa8.into_push_pull_output();
        let stepper_c_dir_pin = gpioa.pa9.into_push_pull_output();

        let (mut stp_setp_a_prod, mut stp_setp_a_cons) = ctx.local.qa.split();
        let (mut stp_setp_b_prod, mut stp_setp_b_cons) = ctx.local.qb.split();
        let (mut stp_setp_c_prod, mut stp_setp_c_cons) = ctx.local.qc.split();

        let stepper_controllers = [
            StackableStepperCtrlr::StepperControllerA(StepperCtrlr::new(stepper_a_stp_pin, stepper_a_dir_pin, Default::default(), stp_setp_a_cons)),
            StackableStepperCtrlr::StepperControllerB(StepperCtrlr::new(stepper_b_stp_pin, stepper_b_dir_pin, Default::default(), stp_setp_b_cons)),
            StackableStepperCtrlr::StepperControllerC(StepperCtrlr::new(stepper_c_stp_pin, stepper_c_dir_pin, Default::default(), stp_setp_c_cons)),
        ];

        //Setup tasks
        let mut controller_task = ControllerTask::new([
            LinearStepperMotor::new(LGA201S06_A_UECB_019::new(), stp_setp_a_prod),
            LinearStepperMotor::new(LGA201S06_A_UECB_019::new(), stp_setp_b_prod),
            LinearStepperMotor::new(LGA201S06_A_UECB_019::new(), stp_setp_c_prod)]);


        // Setup timers
        let mut stepper_main_timer = ctx.device.TIM2.counter_hz(&clocks);
        let mut stepper_second_timer = ctx.device.TIM3.counter_us(&clocks);
        stepper_main_timer.start(STEP_BASE_FREQ.Hz()).unwrap();
        stepper_main_timer.listen(Event::Update);

        stepper_second_timer.listen(Event::Update);
        let mono = Systick::new(ctx.core.SYST, SYSFREQ);
        controller_task_runner::spawn().ok();
        (
            Shared {
                // Initialization of shared resources go here
                stepper_second_timer,
                stepper_controllers,
                steppers_state: Default::default()
            },
            Local {
                // Initialization of local resources go here
                button,
                controller_task,
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
        log::debug!("t:{}",monotonics::now());
        ctx.local.controller_task.run();

        let a:systick_monotonic::fugit::Instant<u64, 1, 1000>  = systick_monotonic::Systick::zero();
        controller_task_runner::spawn_at(a + ExtU64::millis(ctx.local.controller_task.next_run())).unwrap();
    }
    #[task(binds = TIM2, shared = [stepper_controllers, stepper_second_timer, steppers_state], local = [button])]
    fn tim2(mut cx: tim2::Context) {

        // Safe access to local `static mut` variable
        (cx.shared.stepper_controllers, cx.shared.steppers_state).lock(|stepper_controllers, steppers_state| {
            steppers_state[0] = *stepper_controllers[0].run_first_stage();
            steppers_state[1] = *stepper_controllers[1].run_first_stage();
            steppers_state[2] = *stepper_controllers[2].run_first_stage();

            stepper_controllers[0].run_second_stage();
            stepper_controllers[1].run_second_stage();
            stepper_controllers[2].run_second_stage();
        }
        );

        //cx.shared.stepper_second_timer.lock(|stepper_second_timer| {
        //    stepper_second_timer.start(ExtU32::micros(2)).unwrap();
        //}
        //);
    }
    #[task(binds = TIM3, shared = [stepper_controllers, stepper_second_timer])]
    fn tim3(mut cx: tim3::Context) {
        // Safe access to local `static mut` variable
        cx.shared.stepper_second_timer.lock(|stepper_second_timer| {
            stepper_second_timer.cancel().unwrap();

        }
        );
        cx.shared.stepper_controllers.lock(|stepper_controllers| {
            stepper_controllers[0].run_second_stage();

        }
        );
    }
}
