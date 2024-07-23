use bsp_traits::{StepperMotorController, MotorInput, StepperDeviceError, MotorState};
use embedded_time::duration::Microseconds;
use crate::app::button_handler::ButtonHandler;
use crate::app::control_primitives::KinState;
use crate::app::event_queue::get_event_queue;
use crate::app::parameter_manager::{get_parameter_manager, MotorM2Ustep};
pub enum MotorHanderError {
    MotorError(StepperDeviceError, usize),
}

pub struct MotorHandler<'a> {
   motors: [&'a dyn StepperMotorController;3]
}
impl MotorHandler<'_> {
    pub fn new<'a>(motors: [&'a dyn StepperMotorController;3]) -> MotorHandler{
        MotorHandler{
            motors
        }
    }
    pub fn get_motor_state(&mut self) -> Result<[KinState;3], MotorHanderError> {
       let motorM2Ustep=get_parameter_manager().get::<MotorM2Ustep>();
        todo!()
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    use crate::app::event_queue::drain_event_queue;
    use crate::app::parameter_manager::get_parameter_manager;

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