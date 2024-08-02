use core::fmt::Display;
use bsp_traits::{Button, CommsError, MotorEnabler, MotorInput, MotorState, StepperDeviceError, Logger};
use bsp_traits::StepperMotorController;

use crate::boards::{BoardCreationError};
use crate::utils::error_wrapper::ErrorWrapper;
use log;

// global logger
pub struct MyBoard {
}
pub struct InfallibleResources {
    pub motor_enabler: Dummy,
    pub log_device: NativeLogger,
}
pub struct FallibleResources {
    pub button: Dummy,
    pub stp_motor_drive_a: Dummy,
    pub stp_motor_drive_b: Dummy,
    pub stp_motor_drive_c: Dummy,
}

impl MyBoard {
    pub fn new() -> Self {
        MyBoard {}
    }
    pub fn get_infallible_resources(&mut self) -> InfallibleResources {

        InfallibleResources{
            motor_enabler: Dummy{},
            log_device: NativeLogger{},

        }
    }
    pub fn get_fallible_resources(&mut self) -> Result<FallibleResources, BoardCreationError> {
        Ok(FallibleResources{
            button: Dummy{},
            stp_motor_drive_a: Dummy{},
            stp_motor_drive_b: Dummy{},
            stp_motor_drive_c: Dummy{},

        })
    }
}

pub struct Dummy {}
impl MotorEnabler for Dummy {
    fn set_enable(&mut self, enable: bool) {
        todo!()
    }
}
impl StepperMotorController for Dummy {
    fn set_inputs(&mut self, inputs: MotorInput) -> Result<(), StepperDeviceError> {
        todo!()
    }

    fn get_state(&mut self) -> Result<MotorState, StepperDeviceError> {
        todo!()
    }
}
impl Button for Dummy {
    fn is_pressed(&mut self) -> bool {
        todo!()
    }
}
pub struct NativeLogger {}
impl Logger for NativeLogger {

    fn trace(&self, message: &dyn Display) {
        log::trace!("{}", message);
    }

    fn debug(&self, message: &dyn Display) {
        log::debug!("{}", message);
    }

    fn info(&self, message: &dyn Display) {
        log::info!("{}", message);
    }

    fn warn(&self, message: &dyn Display) {
        log::warn!("{}", message);
    }

    fn error(&self, message: &dyn Display) {
        log::error!("{}", message);
    }
}