use core::fmt::Display;
use bsp_traits::{Button, CommsError, MotorEnabler, MotorInput, MotorState, StepperDeviceError, Logger};
use bsp_traits::StepperMotorController;
use crate::boards::{BoardCreationError, BoardResources};
use log;

// global logger
pub struct MockBoard {
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

impl BoardResources for MockBoard{

    type MotorEnabler = Dummy;
    type LogDevice = NativeLogger;
    type Button = Dummy;
    type StepperDriveA = Dummy;
    type StepperDriveB = Dummy;
    type StepperDriveC = Dummy;
    fn get_infallible_resources(&mut self) -> (Self::MotorEnabler, Self::LogDevice){
        (Dummy{}, NativeLogger{})
    }
    fn get_fallible_resources(&mut self) -> Result<
        (Self::Button, Self::StepperDriveA, Self::StepperDriveB, Self::StepperDriveC),
        BoardCreationError> {
        Ok((Dummy{},Dummy{},Dummy{},Dummy{}))
    }
}
impl MockBoard {
    pub fn new() -> Self {
        MockBoard {}
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
        false
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