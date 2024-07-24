#![no_std]
use core::fmt::Arguments;
pub trait BoardSupport {
    fn get_temperature_sensor(&self) -> &dyn TemperatureSensor;
    fn get_stepper_motor_controller(&mut self) -> &dyn StepperMotorController;
    fn get_button(&mut self) -> & mut dyn Button;
}

pub trait TemperatureSensor {
    fn read_temperature(&self) -> Result<f32, DeviceError>;
}
pub trait MotorEnabler {
    fn set_enable(&mut self, enable: bool);
}
#[derive(Debug)]
pub struct MotorState {
    pub velocity: i32,
    pub position: i32,
    pub limit_reached: bool,
}
#[derive(Copy, Clone)]
pub enum MotorMode {
    PositionCtrl,
    VelocityCtrl,
}
pub struct MotorInput {
    pub velocity: i32,
    pub acceleration: i32,
    pub position: i32,
    pub mode: MotorMode,
}
pub trait StepperMotorController {
    fn set_inputs(&mut self, inputs: MotorInput) -> Result<(), StepperDeviceError>;
    fn get_state(&mut self) -> Result<MotorState, StepperDeviceError>;
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


pub trait Logger {
    fn trace(&self, args: Arguments<'_>);
    fn debug(&self, args: Arguments<'_>);
    fn info(&self, args: Arguments<'_>);
    fn warn(&self, args: Arguments<'_>);
    fn error(&self, args: Arguments<'_>);
}
#[derive(Debug)]
pub enum DeviceError {
    CommunicationError,
    InvalidParameter,
    HardwareFailure,
}
#[derive(Debug)]
pub enum StepperDeviceError {
    CommunicationError(CommsError),
    SelfTestVersionMismatch,
    DriverError,
    UnexpectedReset,
    InvalidParameter,
    HardwareFailure,
}
#[derive(Debug)]
pub enum CommsError {
    SPIMutex,
    SPICSPIn,
    SPIOverrun,
    SPICRIC,
    SPIModeFault,
    SPIFrameFormat,
    NotImplemented,
    Unknown,
}
