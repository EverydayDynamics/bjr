use core::sync::atomic::Ordering;
use embedded_hal as hal;
use log;
use crate::stepper_state::{StepperState};
use heapless::spsc::Consumer;
use uom::si::f32::*;
use num_traits::Float;


pub trait StepperCtrlTrait2 {
    fn run(&mut self, current_micros: Time) -> (&StepperState, Time);

}
pub struct StepperCtrlrTask<'a, OP1, OP2>
    where
        OP1: hal::digital::v2::OutputPin,
        OP2: hal::digital::v2::OutputPin
{
    vel: f32,
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
    pub fn new(step_pin: OP1, dir_pin: OP2, setpoint_consumer: Consumer<f32, 2>) ->StepperCtrlrTask<OP1, OP2> {

        log::debug!("stepper created");
        StepperCtrlrTask{
            vel: 0.0,
            setpoint_consumer,
            accel: 0.0,
            dir_pin,
            step_pin,
            last_micros: 0,
            last_step_micros: 0,
            vel_limit: 1000000.0/2.0,
        }
    }
    pub fn run(&mut self, current_micros: u64, state: &StepperState) -> u64 {
        if let Some(received_setpoint) = self.setpoint_consumer.dequeue() {
            self.accel = received_setpoint;
        }
        let mut delta_micros = current_micros as i64 - self.last_micros as i64;


        if delta_micros > 1000000 {
            delta_micros = 1000000;
        } else if delta_micros < 0 {
            delta_micros =0;
        }
        self.vel += (delta_micros as f32 / 1000000.0) * self.accel;
        if self.vel > self.vel_limit {
            self.vel = self.vel_limit;
        }else if self.vel < -self.vel_limit{
            self.vel = -self.vel_limit;
        }
        state.vel.store(self.vel as i32, Ordering::Relaxed);
        let mut micros_until_next_run ;
        if self.vel.abs() > 0.0 {
            let time_between_steps =  1.0 / self.vel.abs();
            let mut micros_since_step = current_micros as i64 - self.last_step_micros as i64;
            if micros_since_step > 10000000 {
                micros_since_step = 10000000
            } else if micros_since_step < 0 {
                micros_since_step =0;
            }
            let micros_until_next_step = (time_between_steps * 1000000.0) as i32 - micros_since_step as i32;
            //log::debug!("m{}",micros_until_next_step);
            if micros_until_next_step > 0 {
                // there is time until next step
                micros_until_next_run = micros_until_next_step;
            } else {
                // STEP NOW!!
                if self.vel > 0.0 {
                    self.dir_pin.set_high().ok().unwrap();
                    state.pos.fetch_add(1, Ordering::Relaxed);
                } else {
                    self.dir_pin.set_low().ok().unwrap();
                    state.pos.fetch_add(-1, Ordering::Relaxed);
                }
                self.step_pin.set_high().ok().unwrap();
                self.last_step_micros = current_micros;
                micros_until_next_run = (time_between_steps * 1000000.0) as i32;
            }
        } else {
            micros_until_next_run = 100;
        }
        if micros_until_next_run > 100 {
            micros_until_next_run = 100;
        }
        if micros_until_next_run < 2 {
            //TODO: schedule missed do some error reporting here
            micros_until_next_run = 2;
        }
        self.last_micros = current_micros;
        //log::debug!("{}",micros_until_next_run);
        self.step_pin.set_low().ok().unwrap();

        return micros_until_next_run as u64;
    }
    pub fn set_internals(&mut self, vel: f32, last_micros: u64, last_step_micros: u64)  {
        self.vel = vel;
        self.last_micros = last_micros;
        self.last_step_micros = last_step_micros;
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

        let mut test_queue: Queue<f32, 2> = Queue::new();
        let (mut test_accel_sender, test_accel_receiver) = test_queue.split();
        let mock_dir = MockOutput::new();
        let mut mock_stp = MockOutput::new();
        let _ = mock_stp.expect_set_high().times(1).returning(||Ok(()));
        let _ = mock_stp.expect_set_low().times(1).returning(||Ok(()));
        let mut test_stepper_state = StepperState { pos: Default::default(), vel: 0.0 };
        let mut tested_stepper = StepperCtrlrTask::new(mock_stp,mock_dir, test_accel_receiver);
        let mut micros = 0u64;
        tested_stepper.set_internals(1000.0,910, 0);
        micros += tested_stepper.run(1001, &test_stepper_state);
    }

    #[test]
    fn test_bad_add() {
    }
}
