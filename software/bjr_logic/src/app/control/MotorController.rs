use crate::app::control_primitives::KinState;
use crate::app::limits::{Limit, LimitLevel, MotorControlVelLimit};
use crate::app::parameter_manager::parameter_manager;
use crate::app::parameter_manager::{MCKp,MCAccel};

pub struct MotorController{
    vel_limit: MotorControlVelLimit,
}
impl Default for MotorController {
    fn default() -> Self {
        MotorController{vel_limit: MotorControlVelLimit::new(LimitLevel::Clamp)}
    }
}
impl MotorController {
    pub fn calc(&mut self, setpoint: KinState, state: KinState) -> KinState {
        let k_p = parameter_manager().get::<MCKp>();
        let accel = parameter_manager().get::<MCAccel>();
        let pos_err = setpoint.pos - state.pos;
        let mut vel_output = pos_err*k_p + setpoint.speed;
        let _ = self.vel_limit.check(&mut vel_output);

       KinState{
           pos: 0.0,
           speed: vel_output,
           accel,
       }
    }
}
