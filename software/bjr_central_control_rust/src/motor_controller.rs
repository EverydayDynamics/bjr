use uom::si::f32::*;
use crate::motor::MotorState;
pub trait MotorController {
    fn run(&self, setpoint: &MotorState, state: &MotorState, feed_forward: &Acceleration) -> Acceleration;
}
pub struct PDPosCtrl {
    p_gain: FrequencyDrift,
    d_gain: Frequency,
}
impl PDPosCtrl {
    pub fn new(p_gain: FrequencyDrift, d_gain: Frequency) -> PDPosCtrl {
        PDPosCtrl  {
            p_gain,
            d_gain,
        }
    }
}
impl MotorController for PDPosCtrl {
    fn run(&self, setpoint: &MotorState, state: &MotorState, feed_forward: &Acceleration) -> Acceleration {
        let pos_error = setpoint.pos - state.pos;
        let vel_error = setpoint.vel - state.vel;
        let output = *feed_forward + pos_error * self.p_gain + vel_error * self.d_gain;
        output
    }
}