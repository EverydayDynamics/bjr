use embedded_hal as hal;
use log;
use crate::stepper_state::{StepperState, STEP_BASE_FREQ,STEP_VELOCITY_SCALER,STEP_THRESHOLD};
use heapless::spsc::Consumer;

pub trait StepperCtrlTrait {
    fn run_first_stage(&mut self) -> &StepperState;
    fn run_second_stage(&mut self);

}
pub struct StepperCtrlr<'a, OP1, OP2>
where
    OP1: hal::digital::v2::OutputPin,
    OP2: hal::digital::v2::OutputPin
{
    state: StepperState,
    setpoint_consumer: Consumer<'a,i32,2>,
    accel: i32,
    ticks_since_last_step: i32,
    step_pin: OP1,
    dir_pin: OP2,
}
impl<OP1, OP2> StepperCtrlr<'_, OP1, OP2>
where
    OP1: hal::digital::v2::OutputPin,
    OP2: hal::digital::v2::OutputPin
{
pub fn new(mut step_pin: OP1, mut dir_pin: OP2, initial_state: StepperState, setpoint_consumer: Consumer<i32, 2>) ->StepperCtrlr<OP1, OP2> {

    log::debug!("stepper created");
    StepperCtrlr{
        state: initial_state,
        setpoint_consumer,
        accel: 0,
        ticks_since_last_step: 0,
        dir_pin,
        step_pin
    }
}
    pub fn set_ticks_since_last_step(&mut self, ticks: i32) {
        self.ticks_since_last_step = ticks;
    }
}
impl<OP1, OP2> StepperCtrlTrait for StepperCtrlr<'_, OP1, OP2>
    where
        OP1: hal::digital::v2::OutputPin,
        OP2: hal::digital::v2::OutputPin
{
    fn run_first_stage(&mut self) -> &StepperState{
        if let Some(received_setpoint) = self.setpoint_consumer.dequeue() {
            self.accel = received_setpoint;
        }
        //self.state.vel += self.accel;
        self.state.vel = 1000000;
        if self.state.vel > STEP_THRESHOLD as i32 {
            self.state.vel = STEP_THRESHOLD as i32;
        } else if self.state.vel < -(STEP_THRESHOLD as i32) {
            self.state.vel = -(STEP_THRESHOLD as i32);
        }
        let a = self.state.vel.abs() * self.ticks_since_last_step;
        if a >= STEP_THRESHOLD {
            if self.state.vel < 0 {
                let a = self.dir_pin.set_high();
                self.state.pos -= 1;
            } else {
                let a = self.dir_pin.set_low();
                self.state.pos += 1;
            }
            let b = self.step_pin.set_high();
            self.ticks_since_last_step = 1;
        } else {
            self.ticks_since_last_step += 1;
            if self.ticks_since_last_step > STEP_THRESHOLD {
                self.ticks_since_last_step = STEP_THRESHOLD;
            }

        }
       &self.state
    }

    fn run_second_stage(&mut self) {
        let a = self.step_pin.set_low();
    }
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::mock_peripherals::MockOutput;
    #[test]
    fn test_creation() {
        let mock_dir = MockOutput::new();
        let mock_stp = MockOutput::new();
        let mut tested_stepper = StepperCtrlr::new(mock_stp,mock_dir,StepperState{ pos: 0, vel: 0 });
        tested_stepper.run_first_stage();
    }

    #[test]
    fn test_bad_add() {
    }
}