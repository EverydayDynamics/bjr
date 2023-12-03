
use rtic::app;
#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1,SPI2])]
mod app {
    use core::sync::atomic::Ordering;
    use stm32f4xx_hal::{timer::{CounterUs, Event},
                        gpio::{gpioa::PA0,
                               gpioa::PA5,
                               Output}, prelude::*};
    use crate::stepper_state::StepperState;
    use crate::controller_task::ControllerTask;
    use crate::actuator_num::NUM_ACTUATOR;
    use crate::captive_linear_stepper::Lga201s06AUecb019;
    use crate::motor::LinearStepperMotor;
    use crate::stepper_driver::SilentStepStick;
    use crate::stepper_controller2::StepperCtrlrTask;
    use systick_monotonic::{fugit::ExtU64, fugit::ExtU32};
    use heapless::spsc::Queue;
    use stm32f4xx_hal::pac;
    use stm32f4xx_hal::timer::MonoTimer64Us;

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
        controller_task: ControllerTask<LinearStepperMotor<'static, Lga201s06AUecb019, SilentStepStick>>,
        stepper_task: StepperCtrlrTask<'static, PA5<Output>,PA0<Output>>,
        stepper_main_timer: CounterUs<stm32f4xx_hal::pac::TIM2>,
        time: u64,
    }
    #[monotonic(binds = TIM3, default = true)]
    type MicrosecMono = MonoTimer64Us<pac::TIM3>;
    #[init(local = [
    qa: Queue<f32, 2> = Queue::new(),
    qb: Queue<f32, 2> = Queue::new(),
    qc: Queue<f32, 2> = Queue::new(),
    qd: Queue<f32, 2> = Queue::new(),
    ])]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_log::init();
        log::debug!("Application started");
        STEPPERS_STATE[0].pos.fetch_add(10,Ordering::Relaxed);
        log::debug!("state:{}",STEPPERS_STATE[0].pos.load(Ordering::Relaxed));
        // syscfg
        let _syscfg = ctx.device.SYSCFG.constrain();
        // clocks
        let rcc = ctx.device.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(SYSFREQ.Hz()).freeze();
        // gpio ports A and C
        let gpioa = ctx.device.GPIOA.split();
        // button
        // led
        let stepper_a_stp_pin = gpioa.pa5.into_push_pull_output();
        let stepper_a_dir_pin = gpioa.pa0.into_push_pull_output();

        let (stp_setp_a_prod, stp_setp_a_cons) = ctx.local.qa.split();
        let (stp_setp_b_prod, _stp_setp_b_cons) = ctx.local.qb.split();
        let (stp_setp_c_prod, _stp_setp_c_cons) = ctx.local.qc.split();

        let stepper_task = StepperCtrlrTask::new(stepper_a_stp_pin, stepper_a_dir_pin,  stp_setp_a_cons);
        //Setup tasks
        let controller_task = ControllerTask::new([
            LinearStepperMotor::new(Lga201s06AUecb019::new(), stp_setp_a_prod, SilentStepStick::new()),
            LinearStepperMotor::new(Lga201s06AUecb019::new(), stp_setp_b_prod, SilentStepStick::new()),
            LinearStepperMotor::new(Lga201s06AUecb019::new(), stp_setp_c_prod, SilentStepStick::new())]);

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
                time: 0,
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

    #[task(binds = TIM2, shared = [], local = [stepper_task, stepper_main_timer, time])]
    fn tim2(ctx: tim2::Context) {
        let micros = monotonics::now().ticks();
        let micros_until_next = ctx.local.stepper_task.run(micros, &STEPPERS_STATE[0]);
        ctx.local.stepper_main_timer.start(ExtU32::micros(micros_until_next as u32)).unwrap();
        *ctx.local.time += micros_until_next ;
    }
}
