use device_traits::{Button, Logger, MotorEnabler, Reader, StepperDeviceError, StepperMotorController, TelemetrySender, TelemetrySenderError, TouchSensor, TouchSensorError};
use core::fmt::{Debug, Display, Formatter};
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
    type MenuIO: core::fmt::Write + Reader;
    type TelemetrySender: TelemetrySender;
    fn get_infallible_resources(&mut self) -> (Self::MotorEnabler, Self::LogDevice, Self::MenuIO);

    fn get_fallible_resources(
        &mut self,
    ) -> Result<
        (
            Self::Button,
            Self::StepperDriveA,
            Self::StepperDriveB,
            Self::StepperDriveC,
            Self::TouchSensor,
            Self::TelemetrySender,
        ),
        BoardCreationError,
    >;
}
pub enum BoardCreationError {
    StepperDriveInitError(StepperDeviceError, usize),
    TouchSensorInitError(TouchSensorError),
    TelemetrySenderInitError(TelemetrySenderError),
}
impl Display for BoardCreationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            BoardCreationError::StepperDriveInitError(err, idx) => {
                write!(
                    f,
                    "BoardCreationError Stepper drive initialization error: {}, idx: {}",
                    err, idx
                )
            }
            BoardCreationError::TouchSensorInitError(err) => {
                write!(
                    f,
                    "BoardCreationError Touch sensor initialization error: {}",
                    err
                )
            }
            BoardCreationError::TelemetrySenderInitError(err) => {
                write!(
                    f,
                    "Telemetry sender initialization error: {}",
                    err
                )

            }
        }
    }
}
impl Debug for BoardCreationError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}
