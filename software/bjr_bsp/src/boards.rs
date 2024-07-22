use bsp_traits::{Button, StepperDeviceError, StepperMotorController, TemperatureSensor};

pub trait BjrBoardSupport {
    fn get_temperature_sensor(&self) -> &dyn TemperatureSensor;
    fn get_stepper_motor_controllers(&mut self) -> [&mut dyn StepperMotorController;1];
    fn get_button(&mut self) -> & mut dyn Button;
}
#[derive(Debug)]
pub enum BoardCreationError{
    StepperDriveInitError(StepperDeviceError, usize)
}
