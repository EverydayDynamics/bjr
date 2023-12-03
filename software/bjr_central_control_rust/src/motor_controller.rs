use crate::motor_state::MotorState;
pub trait MotorController {
    fn run(&self, setpoint: &MotorState, state: &MotorState) -> f32;
}
pub struct PDPosCtrl {
    p_gain: f32, //1/s^2
    d_gain: f32, //1/s
}
impl PDPosCtrl {
    pub fn new(p_gain: f32, d_gain: f32) -> PDPosCtrl {
        PDPosCtrl  {
            p_gain,
            d_gain,
        }
    }
}
impl MotorController for PDPosCtrl {
    fn run(&self, setpoint: &MotorState, state: &MotorState) -> f32 {
        let pos_error = setpoint.pos - state.pos;
        let vel_error = setpoint.vel - state.vel;
        let output = setpoint.accel + pos_error * self.p_gain + vel_error * self.d_gain;
        output
    }
}