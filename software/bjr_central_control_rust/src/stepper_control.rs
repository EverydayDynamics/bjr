use embedded_hal as hal;

pub trait StepperCtrlTrait {
    fn run_first_stage(&mut self, accel_setpoint: Option<i32>);
    fn run_second_stage(&mut self);

}
pub struct StepperCtrlr<OP1, OP2>
where
    OP1: hal::digital::v2::OutputPin,
    OP2: hal::digital::v2::OutputPin
{
    pos: i32,
    vel: i32,
    accel: i32,
    ticks_since_last_step: i32,
    step_pin: OP1,
    dir_pin: OP2,
}
const BASE_FREQ: i32 = 33_000;
const STEP_THRESHOLD: i32 = BASE_FREQ * 1000;
impl<OP1, OP2> StepperCtrlr<OP1, OP2>
where
    OP1: hal::digital::v2::OutputPin,
    OP2: hal::digital::v2::OutputPin
{
pub fn new(mut step_pin: OP1, mut dir_pin: OP2, initial_pos: i32, initial_vel: i32)->StepperCtrlr<OP1, OP2> {
    StepperCtrlr{
        pos: initial_pos,
        vel: initial_vel,
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
impl<OP1, OP2> StepperCtrlTrait for StepperCtrlr<OP1, OP2>
    where
        OP1: hal::digital::v2::OutputPin,
        OP2: hal::digital::v2::OutputPin
{
    fn run_first_stage(&mut self, accel_setpoint: Option<i32>) {
        if let Some(received_setpoint) = accel_setpoint {
            self.accel = received_setpoint;
        }
        self.vel += self.accel;
        if self.vel > STEP_THRESHOLD as i32 {
            self.vel = STEP_THRESHOLD as i32;
        } else if self.vel < -(STEP_THRESHOLD as i32) {
            self.vel = -(STEP_THRESHOLD as i32);
        }
        let a = self.vel.abs() * self.ticks_since_last_step;
        if a > STEP_THRESHOLD {
            if self.vel < 0 {
                let a = self.dir_pin.set_high();
                self.pos -= 1;
            } else {
                self.pos += 1;
            }
            let b = self.step_pin.set_high();
            self.ticks_since_last_step = 1;
        } else {
            self.ticks_since_last_step += 1;
            if self.ticks_since_last_step > STEP_THRESHOLD {
                self.ticks_since_last_step = STEP_THRESHOLD;
            }

        }

    }

    fn run_second_stage(&mut self) {
        let a = self.step_pin.set_low();
        let b = self.dir_pin.set_low();
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
        let mut tested_stepper = StepperCtrlr::new(mock_stp,mock_dir,0,0);
        tested_stepper.run_first_stage(Some(1));
    }

    #[test]
    fn test_bad_add() {
    }
}