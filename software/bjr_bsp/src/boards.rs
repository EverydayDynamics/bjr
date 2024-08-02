use core::fmt::{Debug, Display, Formatter};
use bsp_traits::{Button, Logger, MotorEnabler, StepperDeviceError, StepperMotorController, TemperatureSensor};
use core::result::Result;

pub trait BoardResources {
    // Infallible Resources
    type MotorEnabler;
    type LogDevice;

    // Fallible Resources
    type Button;
    type StepperDriveA;
    type StepperDriveB;
    type StepperDriveC;
    fn get_infallible_resources(&mut self) -> (Self::MotorEnabler, Self::LogDevice);

    fn get_fallible_resources(&mut self) -> Result<
    (Self::Button, Self::StepperDriveA, Self::StepperDriveB, Self::StepperDriveC),
    BoardCreationError>;
}pub enum BoardCreationError{
    StepperDriveInitError(StepperDeviceError, usize)
}
impl Display for BoardCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            BoardCreationError::StepperDriveInitError(err, idx) => {
                write!(f,"BoardCreationError Stepper drive initialization error: {}, idx: {}", err, idx)
            }
        }
    }
}
impl Debug for BoardCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
