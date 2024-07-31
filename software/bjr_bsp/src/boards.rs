use core::fmt::{Debug, Display, Formatter};
use bsp_traits::{Button, Logger, MotorEnabler, StepperDeviceError, StepperMotorController, TemperatureSensor};
pub struct BjrBoardResources<'a>
{
    pub stepper_devicees: Option<[&'a mut dyn StepperMotorController;3]>,
    pub button: Option<&'a mut dyn Button>,
    pub motor_enabler: Option<&'a mut dyn MotorEnabler>,
    pub log_device: Option<&'a mut dyn Logger>,
}
pub trait BjrBoardSupport<'a>
{
    fn get_resources(&'a mut self) -> BjrBoardResources<'a>;
}
pub enum BoardCreationError{
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
