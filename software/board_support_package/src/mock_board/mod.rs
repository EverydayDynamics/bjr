use crate::boards::{BoardCreationError, BoardResources};
use core::fmt::Display;
use device_traits::{
    Button, CommsError, Logger, MotorEnabler, MotorInput, MotorState, Reader, StepperDeviceError,
    TouchSensor, TouchSensorError,
};
use device_traits::{StepperMotorController, TelemetrySender, TelemetrySenderError, TouchPoint};
use log;
use std::io;
use std::io::{Read, Write};
use std::net::{TcpStream};
use std::sync::mpsc::Receiver;
use std::sync::{mpsc};
use std::{thread};

// global logger
pub struct MockBoard {}
pub struct InfallibleResources {
    pub motor_enabler: Dummy,
    pub log_device: NativeLogger,
}
pub struct FallibleResources {
    pub button: Dummy,
    pub stp_motor_drive_a: Dummy,
    pub stp_motor_drive_b: Dummy,
    pub stp_motor_drive_c: Dummy,
}

impl BoardResources for MockBoard {
    type MotorEnabler = Dummy;
    type LogDevice = NativeLogger;
    type Button = Dummy;
    type StepperDriveA = Dummy;
    type StepperDriveB = Dummy;
    type StepperDriveC = Dummy;
    type TouchSensor = Dummy;
    type MenuIO = NativeIO;
    type TelemetrySender = TcpTelemetrySender;

    fn get_infallible_resources(&mut self) -> (Self::MotorEnabler, Self::LogDevice, Self::MenuIO) {
        env_logger::init();
        (Dummy {}, NativeLogger {}, NativeIO::new())
    }
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
    > {
        let telemetry = TcpTelemetrySender::new("127.0.0.1:8080")
            .map_err(|e| BoardCreationError::TelemetrySenderInitError(e))?;
        Ok((Dummy {}, Dummy {}, Dummy {}, Dummy {}, Dummy {}, telemetry))
    }
}
impl MockBoard {
    pub fn new() -> Self {
        MockBoard {}
    }
}

pub struct Dummy {}
impl MotorEnabler for Dummy {
    fn set_enable(&mut self, _enable: bool) {}
}
impl StepperMotorController for Dummy {
    fn set_inputs(&mut self, inputs: MotorInput) -> Result<(), StepperDeviceError> {
        Ok(())
    }

    fn get_state(&mut self) -> Result<MotorState, StepperDeviceError> {
        Ok(MotorState {
            velocity: 0,
            position: 0,
            limit_reached: false,
            velocity_reached: false,
            position_reached: false,
            standstill: false,
        })
    }
    fn set_position(&mut self, new_position: i32) -> Result<(), StepperDeviceError> {
        todo!()
    }

    fn test_motion(&mut self) -> Result<(), StepperDeviceError> {
        todo!()
    }
}
impl Button for Dummy {
    fn is_pressed(&mut self) -> bool {
        false
    }
}
impl TouchSensor for Dummy {
    fn get_touch(&mut self) -> Result<Option<TouchPoint>, TouchSensorError> {
        Ok(None)
    }
}
pub struct NativeLogger {}
impl Logger for NativeLogger {
    fn trace<MSG: Display>(&mut self, message: MSG) {
        log::trace!("{}", message);
    }

    fn debug<MSG: Display>(&mut self, message: MSG) {
        log::debug!("{}", message);
    }

    fn info<MSG: Display>(&mut self, message: MSG) {
        log::info!("{}", message);
    }

    fn warn<MSG: Display>(&mut self, message: MSG) {
        log::warn!("{}", message);
    }

    fn error<MSG: Display>(&mut self, message: MSG) {
        log::error!("{}", message);
    }
}
pub struct NativeIO {
    stdin_channel: Receiver<u8>,
}
impl NativeIO {
    pub fn new() -> NativeIO {
        let stdin_channel = spawn_stdin_channel();
        NativeIO { stdin_channel }
    }
}
fn spawn_stdin_channel() -> Receiver<u8> {
    let (tx, rx) = mpsc::channel::<u8>();
    thread::spawn(move || loop {
        let mut buffer = [0u8; 1];
        io::stdin().read_exact(&mut buffer).unwrap();
        tx.send(buffer[0]).unwrap();
    });
    rx
}

impl Reader for NativeIO {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        let mut received_chars = 0;
        while received_chars < buf.len() {
            if let Ok(key) = self.stdin_channel.try_recv() {
                buf[received_chars] = key;
                received_chars += 1;
            } else {
                break;
            }
        }
        received_chars
    }
}
impl core::fmt::Write for NativeIO {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        io::stdout().write(s.as_ref()).unwrap();
        Ok(())
    }
}
pub struct TcpTelemetrySender {
    address: &'static str,
    stream: Option<TcpStream>,
}

impl TcpTelemetrySender {
    // Constructor
    pub fn new(address: &'static str) -> Result<Self, TelemetrySenderError> {
        Ok(TcpTelemetrySender {
            address,
            stream: None,
        })
    }
}

// Implement the TelemetrySender trait for our struct
impl TelemetrySender for TcpTelemetrySender {
    fn send(&mut self, data: &[u8]) -> Result<(), TelemetrySenderError> {
        if let Some(stream) = &mut self.stream {
            stream
                .write(data)
                .map_err(|_| TelemetrySenderError::SendError)?;
            Ok(())
        } else {
            if let Ok(mut stream) = TcpStream::connect(self.address) {
                stream
                    .write(data)
                    .map_err(|_| TelemetrySenderError::SendError)?;
                self.stream = Some(stream);
                Ok(())
            } else {
                Err(TelemetrySenderError::SendError)
            }
        }
    }
}
