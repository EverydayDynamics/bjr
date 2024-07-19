use crate::bsp::traits::MotorState;
use super::super::traits::{StepperMotorController, DeviceError};

pub(super) struct MyBoardStepperMotorController {
    // Add fields for controller-specific details
}

impl MyBoardStepperMotorController {
    pub(super) fn new() -> Self {
        MyBoardStepperMotorController { /* ... */ }
    }
}

impl StepperMotorController for MyBoardStepperMotorController {

    fn set_run_values(&mut self, speed: i32, accel: i32) -> Result<(), DeviceError> {
        todo!()
    }

    fn get_state(&self) -> Result<MotorState, DeviceError> {
        todo!()
    }
}