use core::sync::atomic::Ordering;
use embedded_hal as hal;
use log;
use crate::stepper_state::{StepperState};
use heapless::spsc::Consumer;
use uom::si::f32::*;
use uom::si::frequency_drift::hertz_per_second;
use uom::si::time::microsecond;
use uom::si::frequency::hertz;
use uom::si::ratio::ratio;
use num_traits::Float;


pub trait StepperCtrlTrait2 {
    fn run(&mut self, current_micros: Time) -> (&StepperState, Time);

}
pub struct StepperCtrlrTask<'a, OP1, OP2>
    where
        OP1: hal::digital::v2::OutputPin,
        OP2: hal::digital::v2::OutputPin
{
    state: StepperState,
    setpoint_consumer: Consumer<'a,f32,2>,
    accel: f32,
    step_pin: OP1,
    dir_pin: OP2,
    last_micros: u64,
    last_step_micros: u64,
    vel_limit: f32,
}

impl<OP1, OP2> StepperCtrlrTask<'_, OP1, OP2>
    where
        OP1: hal::digital::v2::OutputPin,
        OP2: hal::digital::v2::OutputPin
{
    pub fn new(step_pin: OP1, dir_pin: OP2, initial_state: StepperState, setpoint_consumer: Consumer<f32, 2>) ->StepperCtrlrTask<OP1, OP2> {

        log::debug!("stepper created");
        StepperCtrlrTask{
            state: initial_state,
            setpoint_consumer,
            accel: 0.0,
            dir_pin,
            step_pin,
            last_micros: 0,
            last_step_micros: 0,
            vel_limit: 30000.0,
        }
    }
    pub fn run(&mut self, current_micros: u64, state: &StepperState) -> (u64) {
        if let Some(received_setpoint) = self.setpoint_consumer.dequeue() {
            self.accel = received_setpoint;
        }
        let mut delta_micros = current_micros - self.last_micros;

        if delta_micros > 1000000 {
            delta_micros = 1000000;
        }
        self.state.vel += (delta_micros as f32 / 1000000.0) * self.accel;
        if self.state.vel > self.vel_limit {
            self.state.vel = self.vel_limit;
        }else if self.state.vel < -self.vel_limit{
            self.state.vel = -self.vel_limit;
        }

        self.state.vel = 4000.0;
        let mut micros_until_next_run = 0;
        if self.state.vel.abs() > 0.1 {
            let time_between_steps =  1.0 / self.state.vel.abs();
            let mut micros_since_step = current_micros - self.last_step_micros;
            if micros_since_step > 10000000 {
                micros_since_step = 10000000
            }
            let micros_until_next_step = (time_between_steps * 1000000.0) as i32 - micros_since_step as i32;
            if micros_until_next_step > 0 {
                // there is time until next step
                micros_until_next_run = micros_until_next_step;
            } else {
                // STEP NOW!!
                if self.state.vel < 0.0 {
                    let a = self.dir_pin.set_high();
                    state.pos.fetch_add(-1, Ordering::Relaxed);
                } else {
                    let a = self.dir_pin.set_low();
                    state.pos.fetch_add(1, Ordering::Relaxed);
                }
                let b = self.step_pin.set_high();
                self.last_step_micros = current_micros;
                micros_until_next_run = (time_between_steps * 1000000.0) as i32;
            }
        } else {
            micros_until_next_run = 100;
        }
        if micros_until_next_run > 100 {
            micros_until_next_run = 100;
        }
        self.last_micros = current_micros;
        //log::debug!("{}",micros_until_next_run);
        let b = self.step_pin.set_low();

        micros_until_next_run as u64
    }
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::mock_peripherals::MockOutput;
    use heapless::spsc::Queue;
    #[test]
    fn test_creation() {
        let mut test_queue: Queue<FrequencyDrift2> = Queue::new();
        let (mut test_accel_sender, test_accel_receiver) = test_queue.split();
        let mock_dir = MockOutput::new();
        let mock_stp = MockOutput::new();
        let mut tested_stepper = StepperCtrlrTask::new(mock_stp,mock_dir,StepperState{ pos: 0, vel: 0 }, test_accel_receiver);
        let mut micros = 0u64;
        let mut state_result: StepperState = Default::default();
        _ = test_accel_sender.enqueue(100);
        (_, micros) = tested_stepper.run(micros);
        (_, micros) = tested_stepper.run(micros);
        (_, micros) = tested_stepper.run(micros);
    }

    #[test]
    fn test_bad_add() {
    }
}
