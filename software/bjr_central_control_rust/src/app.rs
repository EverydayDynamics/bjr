
use rtic::app;
#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI2,SPI1])]
mod app {
    use core::sync::atomic::Ordering;
    use stm32f4xx_hal::{timer::{CounterUs, Event},
                        gpio::{gpioa::PA0,
                               gpioa::PA5,
                               gpioa::PA1,
                               gpioa::PA4,
                               gpioa::PA6,
                               gpioa::PA7,
                               gpioa::PA8,
                               gpioa::PA9,
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
    use systick_monotonic::Systick;

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
        stepper_task_a: StepperCtrlrTask<'static, PA5<Output>,PA0<Output>>,
        stepper_task_b: StepperCtrlrTask<'static, PA6<Output>,PA7<Output>>,
        stepper_task_c: StepperCtrlrTask<'static, PA8<Output>,PA9<Output>>,
        stepper_main_timer: CounterUs<stm32f4xx_hal::pac::TIM2>,
        time: u64,
        debug_pin: PA1<Output>,
        debug_pin2: PA4<Output>,
    }
    #[monotonic(binds = TIM3, default = true)]
    type MicrosecMono = MonoTimer64Us<pac::TIM3>;
    //type MicrosecMono = Systick<1000000>;
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

        let stepper_b_stp_pin = gpioa.pa6.into_push_pull_output();
        let stepper_b_dir_pin = gpioa.pa7.into_push_pull_output();

        let stepper_c_stp_pin = gpioa.pa8.into_push_pull_output();
        let stepper_c_dir_pin = gpioa.pa9.into_push_pull_output();

        let mut debug_pin = gpioa.pa1.into_push_pull_output();
        let mut debug_pin2 = gpioa.pa4.into_push_pull_output();

        let (stp_setp_a_prod, stp_setp_a_cons) = ctx.local.qa.split();
        let (stp_setp_b_prod, stp_setp_b_cons) = ctx.local.qb.split();
        let (stp_setp_c_prod, stp_setp_c_cons) = ctx.local.qc.split();

        let stepper_task_a = StepperCtrlrTask::new(stepper_a_stp_pin, stepper_a_dir_pin,  stp_setp_a_cons);
        let stepper_task_b = StepperCtrlrTask::new(stepper_b_stp_pin, stepper_b_dir_pin,  stp_setp_b_cons);
        let stepper_task_c = StepperCtrlrTask::new(stepper_c_stp_pin, stepper_c_dir_pin,  stp_setp_c_cons);
        //Setup tasks
        let controller_task = ControllerTask::new([
            LinearStepperMotor::new(Lga201s06AUecb019::new(), stp_setp_a_prod, SilentStepStick::new()),
            LinearStepperMotor::new(Lga201s06AUecb019::new(), stp_setp_b_prod, SilentStepStick::new()),
            LinearStepperMotor::new(Lga201s06AUecb019::new(), stp_setp_c_prod, SilentStepStick::new())]);

        // Setup timers
        let mut stepper_main_timer = ctx.device.TIM2.counter_us(&clocks);
        stepper_main_timer.start(ExtU32::micros(1000)).unwrap();
        stepper_main_timer.listen(Event::Update);

        //let mono = Systick::new(ctx.core.SYST, 1000000);
        let mono = ctx.device.TIM3.monotonic64_us(&clocks);
        controller_task_runner::spawn().ok();
        //tim2_task::spawn().ok();
        (
            Shared {
                // Initialization of shared resources go here
            },
            Local {
                // Initialization of local resources go here
                controller_task,
                stepper_task_a,
                stepper_task_b,
                stepper_task_c,
                stepper_main_timer,
                time: 0,
                debug_pin,
                debug_pin2,
            },
            init::Monotonics(mono),
        )
    }

    #[task(local = [controller_task, debug_pin2], shared = [], priority = 1)]
    fn controller_task_runner(ctx: controller_task_runner::Context) {
        let _ = ctx.local.debug_pin2.set_high();
        ctx.local.controller_task.update_stepper_state(&STEPPERS_STATE);
        ctx.local.controller_task.run();

        let a:systick_monotonic::fugit::Instant<u64, 1, 1000000>  = systick_monotonic::Systick::zero();
        controller_task_runner::spawn_at(a + ExtU64::micros(ctx.local.controller_task.next_run())).unwrap();
        let _ = ctx.local.debug_pin2.set_low();
    }

    #[task(binds=TIM2, shared = [], local = [stepper_task_a, stepper_task_b,stepper_task_c, stepper_main_timer, time, debug_pin],priority = 2)]
    fn tim2(mut ctx: tim2::Context) {
        let _ = ctx.local.debug_pin.set_high();
        ctx.local.stepper_main_timer.cancel().unwrap();
        let micros = *ctx.local.time;
        let mut micros_until_next = ctx.local.stepper_task_a.run(micros, &STEPPERS_STATE[0]);
        let micros_until_next_b = ctx.local.stepper_task_b.run(micros, &STEPPERS_STATE[1]);
        if micros_until_next_b < micros_until_next {
            micros_until_next = micros_until_next_b;
        }
        let micros_until_next_c = ctx.local.stepper_task_c.run(micros, &STEPPERS_STATE[2]);
        if micros_until_next_c < micros_until_next {
            micros_until_next = micros_until_next_c;
        }
        ctx.local.stepper_main_timer.start(ExtU32::micros((micros_until_next-micros-6 )as u32)).unwrap();
        //let a:systick_monotonic::fugit::Instant<u64, 1, 1000000>  = systick_monotonic::Systick::zero();
        //tim2_task::spawn_at(a + ExtU64::micros(micros_until_next)).unwrap();
        *ctx.local.time = micros_until_next;
        ctx.local.stepper_task_a.reset_step_pin();
        ctx.local.stepper_task_b.reset_step_pin();
        ctx.local.stepper_task_c.reset_step_pin();
        let _ = ctx.local.debug_pin.set_low();
    }
}
