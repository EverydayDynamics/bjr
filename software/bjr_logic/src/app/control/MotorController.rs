use crate::app::control_primitives::KinState;
use crate::app::limits::{Limit, LimitLevel, MotorControlVelLimit};
use crate::app::parameter_manager::{MCPosDeadBand, parameter_manager};
use crate::app::parameter_manager::{MCKp,MCAccel};
use libm::fabsf;

pub struct MotorController{
    vel_limit: MotorControlVelLimit,
}
impl Default for MotorController {
    fn default() -> Self {
        MotorController{vel_limit: MotorControlVelLimit::new(LimitLevel::Clamp)}
    }
}
impl MotorController {
    pub fn calc(&mut self, setpoint: KinState, state: KinState) -> (KinState, f32) {
        let k_p = parameter_manager().get::<MCKp>();
        let accel = parameter_manager().get::<MCAccel>();
        let pos_deadband = parameter_manager().get::<MCPosDeadBand>();
        let mut pos_err = setpoint.pos - state.pos;
        if fabsf(pos_err) < pos_deadband {
            pos_err = 0.0;
        } else {

        }
        let mut vel_output = pos_err*k_p + setpoint.speed;
        let _ = self.vel_limit.check(&mut vel_output);

        (KinState{
           pos: 0.0,
           speed: vel_output,
           accel,
       },pos_err)
    }
}
