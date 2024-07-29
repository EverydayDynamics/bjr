use core::fmt::{Display, Formatter};
use bsp_traits::{Button, Logger, MotorEnabler, StepperDeviceError, StepperMotorController, TemperatureSensor};
pub struct BjrBoardResources<'a> {
    pub stepper_devices: Option<[&'a mut dyn StepperMotorController;3]>,
    pub button: Option<&'a mut dyn Button>,
    pub motor_enabler: Option<&'a mut dyn MotorEnabler>,
    pub log_device: Option<&'a mut dyn Logger>,
}
pub trait BjrBoardSupport
{
    fn get_stepper_motor_controller_a(&mut self) -> impl StepperMotorController;
    fn get_stepper_motor_controller_b(&mut self) -> impl StepperMotorController;
    fn get_stepper_motor_controller_c(&mut self) -> impl StepperMotorController;
    fn get_button(& self) -> impl Button;
    fn get_motor_enabler(&mut self) -> impl MotorEnabler;
    fn get_log_device(&mut self) -> impl Logger;
    fn get_resources(&mut self) -> BjrBoardResources;
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
