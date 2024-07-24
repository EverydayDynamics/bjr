use bsp_traits::{StepperMotorController, StepperDeviceError};
use crate::app::control_primitives::KinState;
use crate::app::limits::{LimitError, MotorPosLimit, MotorVelLimit, Limit};
use crate::app::motor_handler::MotorHandlerError::MotorError;
use crate::app::parameter_manager::{parameter_manager, MotorM2Ustep};

const MOTOR_NUM:usize = 3;
pub enum MotorHandlerError {
    MotorError(StepperDeviceError, usize),
    MotorPositionLimitError(LimitError<f32>, usize),
    MotorVelocityLimitError(LimitError<f32>, usize),
}

pub struct MotorHandler<'a> {
   motors: [&'a mut dyn StepperMotorController;MOTOR_NUM],
    accel: [f32;MOTOR_NUM],
    poslim: MotorPosLimit,
    vellim: MotorVelLimit,
}
impl MotorHandler<'_> {
    pub fn new(motors: [&mut dyn StepperMotorController;MOTOR_NUM], poslim: MotorPosLimit, vellim: MotorVelLimit) -> MotorHandler{
        MotorHandler{
            motors,
            accel: [0.0f32;3],
            poslim,
            vellim,
        }
    }
    pub fn get_motor_state(&mut self) -> Result<[KinState;MOTOR_NUM], MotorHandlerError> {
       let mut state:[KinState;MOTOR_NUM] = Default::default();
        let motm2us = parameter_manager().get::<MotorM2Ustep>();
        for motor_idx in 0..MOTOR_NUM {
            let motstate = self.motors[motor_idx].get_state().map_err(|e|MotorHandlerError::MotorError(e, motor_idx))?;
            state[motor_idx].accel = self.accel[motor_idx];
            state[motor_idx].speed = (motstate.velocity as f64 *motm2us as f64) as f32;
            state[motor_idx].pos = (motstate.position as f64 *motm2us as f64) as f32;
            self.vellim.check(state[motor_idx].speed).map_err(|e|MotorHandlerError::MotorVelocityLimitError(e, motor_idx))?;
            self.poslim.check(state[motor_idx].pos).map_err(|e|MotorHandlerError::MotorPositionLimitError(e, motor_idx))?;
        }
        Ok(state)
    }

}

#[cfg(test)]
mod tests {
    use bsp_traits::{MotorInput, MotorState};
use super::*;
    use mockall::mock;
    use crate::app::event_queue::drain_event_queue;
    use crate::app::parameter_manager::parameter_manager;

    mock! {
        pub StepperMotorController {}
        impl StepperMotorController for StepperMotorController {
            fn set_inputs(&mut self, inputs: MotorInput) -> Result<(), StepperDeviceError>;
            fn get_state(&mut self) -> Result<MotorState, StepperDeviceError>;
        }
    }

    #[test]
    fn test_motors() {

    }
}