use embedded_hal as hal;

use hal::prelude::*;
pub trait StepperCtrlTrait {
    fn run_first_stage(accel_setpoint: Option<i32>);
    fn run_second_stage();

}
pub struct StepperCtrlr<OP1, OP2>
where
    OP1: hal::digital::v2::OutputPin,
    OP2: hal::digital::v2::OutputPin
{
    pos: i32,
    vel: i32,
    accel: i32,
    step_pin: OP1,
    dir_pin: OP2,
}
const BASE_FREQ: u32 = 33_000;
impl<OP1, OP2> StepperCtrlr<OP1, OP2>
where
    OP1: hal::digital::v2::OutputPin,
    OP2: hal::digital::v2::OutputPin
{
pub fn new(mut step_pin: OP1, mut dir_pin: OP2, initial_pos: i32, initial_vel: i32 )->StepperCtrlr<OP1, OP2> {
    StepperCtrlr{
        pos: initial_pos,
        vel: initial_vel,
        accel: 0,
        dir_pin,
        step_pin
    }
}
}
impl<OP1, OP2> StepperCtrlTrait for StepperCtrlr<OP1, OP2>
    where
        OP1: hal::digital::v2::OutputPin,
        OP2: hal::digital::v2::OutputPin
{
    fn run_first_stage(accel_setpoint: Option<i32>) {
    }

    fn run_second_stage() {
    }
}
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// This is a really bad adding function, its purpose is to fail in this
// example.
#[allow(dead_code)]
fn bad_add(a: i32, b: i32) -> i32 {
    a - b
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }

    #[test]
    fn test_bad_add() {
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(bad_add(1, 2), 3);
    }
}