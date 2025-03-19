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
    fn set_position(&mut self, new_position: i32) -> Result<(), StepperDeviceError>;
    fn test_motion(&mut self) -> Result<(), StepperDeviceError>;
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

impl core::fmt::Display for TouchSensorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TouchSensorError::CommunicationError(e) => write!(f, "Communication error: {}", e),
        }
    }
}
pub trait Button {
    fn is_pressed(&mut self) -> bool;
}

pub trait Monotonic {
    fn current_time(&mut self) -> u64;
}
pub trait Reader {
    fn read(&mut self, buf: &mut [u8]) -> usize;
}
pub trait LoggableMessage: core::fmt::Display {}
pub trait Logger {
    fn trace<MSG: LoggableMessage>(&mut self, message: MSG);
    fn debug<MSG: LoggableMessage>(&mut self, message: MSG);
    fn info<MSG: LoggableMessage>(&mut self, message: MSG);
    fn warn<MSG: LoggableMessage>(&mut self, message: MSG);
    fn error<MSG: LoggableMessage>(&mut self, message: MSG);
}
pub enum DeviceError {
    CommunicationError,
    InvalidParameter,
    HardwareFailure,
}
#[derive(PartialEq, Copy, Clone)]
pub enum StepperMotorPhase {
    A,
    B,
}
#[cfg(not(feature = "defmt"))]
impl Display for StepperMotorPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            StepperMotorPhase::A => {
                write!(f, "A")
            }
            StepperMotorPhase::B => {
                write!(f, "B")
            }
        }
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for StepperMotorPhase {
    fn format(&self, f: defmt::Formatter) {
        match self {
            StepperMotorPhase::A => {
                defmt::write!(f, "A")
            }
            StepperMotorPhase::B => {
                defmt::write!(f, "B")
            }
        }
    }
}
#[derive(PartialEq, Copy, Clone)]
pub enum StepperDeviceError {
    CommunicationError(CommsError),
    SelfTestVersionMismatch,
    DriverError,
    ShortToGround(StepperMotorPhase),
    OpenLoad(StepperMotorPhase),
    OverTemperatureShutdown,
    UnexpectedReset,
    InvalidParameter,
    EnableError,
    HardwareFailure,
}
#[cfg(not(feature = "defmt"))]
impl Display for StepperDeviceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            StepperDeviceError::CommunicationError(e) => {
                write!(f, "Communication error: {}", e)
            }
            StepperDeviceError::SelfTestVersionMismatch => {
                write!(f, "Self Test version mismatch")
            }
            StepperDeviceError::DriverError => {
                write!(f, "driver error")
            }
            StepperDeviceError::UnexpectedReset => {
                write!(f, "Unexpected Reset")
            }
            StepperDeviceError::InvalidParameter => {
                write!(f, "Invalid Parameter")
            }
            StepperDeviceError::HardwareFailure => {
                write!(f, "Hardware Failure")
            }
            StepperDeviceError::EnableError => {
                write!(f, "Enable Error")
            }
            StepperDeviceError::ShortToGround(ph) => {
                write!(f, "Phase {} shorted to ground", ph)
            }
            StepperDeviceError::OverTemperatureShutdown => {
                write!(f, "Over temperature shutdown")
            }
            StepperDeviceError::OpenLoad(ph) => {
                write!(f, "Phase {} open load", ph)
            }
        }
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for StepperDeviceError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            StepperDeviceError::CommunicationError(e) => {
                defmt::write!(f, "Communication error: {}", e)
            }
            StepperDeviceError::SelfTestVersionMismatch => {
                defmt::write!(f, "Self Test version mismatch")
            }
            StepperDeviceError::DriverError => {
                defmt::write!(f, "driver error")
            }
            StepperDeviceError::UnexpectedReset => {
                defmt::write!(f, "Unexpected Reset")
            }
            StepperDeviceError::InvalidParameter => {
                defmt::write!(f, "Invalid Parameter")
            }
            StepperDeviceError::HardwareFailure => {
                defmt::write!(f, "Hardware Failure")
            }
            StepperDeviceError::ShortToGround(ph) => {
                defmt::write!(f, "Phase {} shorted to ground", ph)
            }
            StepperDeviceError::OverTemperatureShutdown => {
                defmt::write!(f, "Over temperature shutdown")
            }
            StepperDeviceError::OpenLoad(ph) => {
                defmt::write!(f, "Phase {} open load", ph)
            }
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
#[cfg(not(feature = "defmt"))]
impl Display for CommsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            CommsError::SPIMutex => {
                write!(f, "CommsError SPIMutex")
            }
            CommsError::SPICSPIn => {
                write!(f, "CommsError SPICSPIn")
            }
            CommsError::SPIOverrun => {
                write!(f, "CommsError SPIOverrun")
            }
            CommsError::SPICRIC => {
                write!(f, "CommsError SPICRIC")
            }
            CommsError::SPIModeFault => {
                write!(f, "CommsError SPIModeFault")
            }
            CommsError::SPIFrameFormat => {
                write!(f, "CommsError SPIFrameFormat")
            }
            CommsError::NotImplemented => {
                write!(f, "CommsError NotImplemented")
            }
            CommsError::Unknown => {
                write!(f, "CommsError Unknown")
            }
        }
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for CommsError {
    fn format(&self, f: defmt::Formatter) {
        match self {
            CommsError::SPIMutex => {
                defmt::write!(f, "CommsError SPIMutex")
            }
            CommsError::SPICSPIn => {
                defmt::write!(f, "CommsError SPICSPIn")
            }
            CommsError::SPIOverrun => {
                defmt::write!(f, "CommsError SPIOverrun")
            }
            CommsError::SPICRIC => {
                defmt::write!(f, "CommsError SPICRIC")
            }
            CommsError::SPIModeFault => {
                defmt::write!(f, "CommsError SPIModeFault")
            }
            CommsError::SPIFrameFormat => {
                defmt::write!(f, "CommsError SPIFrameFormat")
            }
            CommsError::NotImplemented => {
                defmt::write!(f, "CommsError NotImplemented")
            }
            CommsError::Unknown => {
                defmt::write!(f, "CommsError Unknown")
            }
        }
    }
}
#[derive(Clone, Copy)]
pub enum TelemetrySenderError {
    ConnectionError,
    SendError,
}
impl Display for TelemetrySenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            TelemetrySenderError::ConnectionError => {write!(f, "Connection Error")}
            TelemetrySenderError::SendError => {write!(f, "Sending Error")}
        }
    }
}
pub trait TelemetrySender {
    fn send(&mut self, data:&[u8])-> Result<(),TelemetrySenderError>;
}