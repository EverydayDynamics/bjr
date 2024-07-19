pub trait BoardSupport {
    fn get_temperature_sensor(&self) -> &dyn TemperatureSensor;
    fn get_stepper_motor_controller(&self) -> &dyn StepperMotorController;
    fn get_button(&mut self) -> & mut dyn Button;
}

pub trait TemperatureSensor {
    fn read_temperature(&self) -> Result<f32, DeviceError>;
}
pub trait MotorEnabler {
    fn set_enable(&mut self, enable: bool);
}
pub struct MotorState {
    pub velocity: i32,
    pub position: i32,
}
pub trait StepperMotorController {

    fn set_run_values(&mut self, speed: i32, accel: i32) -> Result<(), DeviceError>;
    fn get_state(&self) -> Result<MotorState, DeviceError>;
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
    fn current_time(&mut self) -> u64;
}
#[derive(Debug)]
pub enum DeviceError {
    CommunicationError,
    InvalidParameter,
    HardwareFailure,
}