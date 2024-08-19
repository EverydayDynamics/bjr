use crate::boards::{BoardCreationError, BoardResources};
use bsp_traits::StepperMotorController;
use bsp_traits::{
    Button, CommsError, Logger, MotorEnabler, MotorInput, MotorState, Point, Reader,
    StepperDeviceError, TouchSensor, TouchSensorError,
};
use core::fmt::Display;
use log;
use std::io;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};

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
        ),
        BoardCreationError,
    > {
        Ok((Dummy {}, Dummy {}, Dummy {}, Dummy {}, Dummy {}))
    }
}
impl MockBoard {
    pub fn new() -> Self {
        MockBoard {}
    }
}

pub struct Dummy {}
impl MotorEnabler for Dummy {
    fn set_enable(&mut self, enable: bool) {}
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
}
impl Button for Dummy {
    fn is_pressed(&mut self) -> bool {
        false
    }
}
impl TouchSensor for Dummy {
    fn get_touch(&mut self) -> Result<Option<Point>, TouchSensorError> {
        Ok(None)
    }
}
pub struct NativeLogger {}
impl Logger for NativeLogger {
    fn trace(&mut self, message: &dyn Display) {
        log::trace!("{}", message);
    }

    fn debug(&mut self, message: &dyn Display) {
        log::debug!("{}", message);
    }

    fn info(&mut self, message: &dyn Display) {
        log::info!("{}", message);
    }

    fn warn(&mut self, message: &dyn Display) {
        log::warn!("{}", message);
    }

    fn error(&mut self, message: &dyn Display) {
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
