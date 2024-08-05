use core::fmt::{Debug, Display, Formatter};
use bsp_traits::{Button, Logger, MotorEnabler, StepperDeviceError, StepperMotorController, TemperatureSensor, TouchSensor, TouchSensorError};
use core::result::Result;

pub trait BoardResources {
    // Infallible Resources
    type MotorEnabler: MotorEnabler;
    type LogDevice: Logger;

    // Fallible Resources
    type Button: Button;
    type StepperDriveA: StepperMotorController;
    type StepperDriveB: StepperMotorController;
    type StepperDriveC: StepperMotorController;
    type TouchSensor: TouchSensor;
    fn get_infallible_resources(&mut self) -> (Self::MotorEnabler, Self::LogDevice);

    fn get_fallible_resources(&mut self) -> Result<
    (Self::Button,
     Self::StepperDriveA,
     Self::StepperDriveB,
     Self::StepperDriveC,
     Self::TouchSensor,
    ),
    BoardCreationError>;
}pub enum BoardCreationError{
    StepperDriveInitError(StepperDeviceError, usize),
    TouchSensorInitError(TouchSensorError),
}
impl Display for BoardCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            BoardCreationError::StepperDriveInitError(err, idx) => {
                write!(f,"BoardCreationError Stepper drive initialization error: {}, idx: {}", err, idx)
            }
            BoardCreationError::TouchSensorInitError(err) => {
                write!(f,"BoardCreationError Touch sensor initialization error: {}", err)
            }
        }
    }
}
impl Debug for BoardCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
