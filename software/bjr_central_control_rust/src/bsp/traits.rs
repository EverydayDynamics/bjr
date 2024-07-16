pub trait BoardSupport {
    fn get_temperature_sensor(&self) -> &dyn TemperatureSensor;
    fn get_stepper_motor_controller(&self) -> &dyn StepperMotorController;
    fn get_button(&mut self) -> & mut dyn Button;
}

pub trait TemperatureSensor {
    fn read_temperature(&self) -> Result<f32, DeviceError>;
}

pub trait StepperMotorController {
    fn set_speed(&mut self, speed: u32) -> Result<(), DeviceError>;
    fn move_steps(&mut self, steps: i32) -> Result<(), DeviceError>;
    fn get_position(&self) -> Result<i32, DeviceError>;
}

pub struct Point {
    pub x: f32,
    pub y: f32,
}
pub trait TouchSensor {
    fn get_touch(&mut self) -> Result<Option<Point>, DeviceError>;
}

pub trait Button{
    fn is_pressed(&mut self) -> bool;
}

pub trait Monotonic {
    fn current_instance(&mut self) -> u64;
}
#[derive(Debug)]
pub enum DeviceError {
    CommunicationError,
    InvalidParameter,
    HardwareFailure,
}