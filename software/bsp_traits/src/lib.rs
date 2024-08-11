#![no_std]
use core::fmt::{Display, Formatter};
pub trait TemperatureSensor {
    fn read_temperature(&self) -> Result<f32, DeviceError>;
}
pub trait MotorEnabler {
    fn set_enable(&mut self, enable: bool);
}
pub struct MotorState {
    pub velocity: i32,
    pub position: i32,
    pub limit_reached: bool,
    pub velocity_reached: bool,
    pub position_reached: bool,
    pub standstill: bool,
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
    fn set_position(&mut self, new_position:i32) -> Result<(), StepperDeviceError>;
}

pub struct Point {
    pub x: i32,
    pub y: i32,
}
pub trait TouchSensor {
    fn get_touch(&mut self) -> Result<Option<Point>, TouchSensorError>;
}
#[derive(PartialEq, Copy, Clone)]
pub enum TouchSensorError {
    CommunicationError(CommsError),
}
impl Display for TouchSensorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            TouchSensorError::CommunicationError(e) => {write!(f,"TouchSensor Communication error: {}", e)}
        }
    }
}

pub trait Button{
    fn is_pressed(&mut self) -> bool;
}

pub trait Monotonic {
    fn current_time(&mut self) -> u64;
}
pub trait Reader {
    fn read(&mut self, buf: &mut [u8]) -> usize;
}

pub trait Logger {
    fn trace(&self, message: &dyn Display);
    fn debug(&self, message: &dyn Display);
    fn info(&self, message: &dyn Display);
    fn warn(&self, message: &dyn Display);
    fn error(&self, message: &dyn Display);
}
pub enum DeviceError {
    CommunicationError,
    InvalidParameter,
    HardwareFailure,
}
#[derive(PartialEq, Copy, Clone)]
pub enum StepperDeviceError {
    CommunicationError(CommsError),
    SelfTestVersionMismatch,
    DriverError,
    UnexpectedReset,
    InvalidParameter,
    HardwareFailure,
}
impl Display for StepperDeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            StepperDeviceError::CommunicationError(e) => {write!(f,"StepperDevice Communication error: {}", e)}
            StepperDeviceError::SelfTestVersionMismatch => {write!(f,"StepperDevice Self Test Failure")}
            StepperDeviceError::DriverError => {write!(f,"StepperDevice driver error")}
            StepperDeviceError::UnexpectedReset => {write!(f,"StepperDevice Unexpected Reset")}
            StepperDeviceError::InvalidParameter => {write!(f,"StepperDevice Invalid Parameter")}
            StepperDeviceError::HardwareFailure => {write!(f,"StepperDevice Hardware Failure")}
        }
    }
}
#[derive(PartialEq, Copy, Clone)]
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
impl Display for CommsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            CommsError::SPIMutex => {write!(f,"CommsError SPIMutex")}
            CommsError::SPICSPIn => {write!(f,"CommsError SPICSPIn")}
            CommsError::SPIOverrun => {write!(f,"CommsError SPIOverrun")}
            CommsError::SPICRIC => {write!(f,"CommsError SPICRIC")}
            CommsError::SPIModeFault => {write!(f,"CommsError SPIModeFault")}
            CommsError::SPIFrameFormat => {write!(f,"CommsError SPIFrameFormat")}
            CommsError::NotImplemented => {write!(f,"CommsError NotImplemented")}
            CommsError::Unknown => {write!(f,"CommsError Unknown")}
        }
    }
}