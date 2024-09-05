use crate::boards::{BoardCreationError, BoardResources};
use device_traits::StepperMotorController;
use device_traits::{
    Button, CommsError, MotorEnabler, MotorInput, MotorState, Reader, StepperDeviceError,
};
use core::cell::RefCell;
use cortex_m::interrupt;
use cortex_m::interrupt::Mutex;
use stm32f4xx_hal as hal;
use stm32f4xx_hal::gpio::{Output, Pin, PinState};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::spi::{Phase, Polarity, Spi};

use crate::devices::gpio_motor_enabler::GPIOMotorEnabler;
use crate::devices::rtt_logger::RttLogger;
use crate::devices::rtt_rw_interface::RttRWInterface;
use crate::devices::tsc2046_touchscreen_dev::Tsc2046TouchDev;
use crate::devices::{gpio_button::GpioButton, tmc5130_stepper_dev::TMC5130StepperDev};
use crate::utils::error_wrapper::ErrorWrapper;
use crate::utils::spidev::{SpiDevError, Spidev};
use embedded_hal::spi::{Error, ErrorKind};
use rtt_target::{rtt_init, ChannelMode, UpChannel, DownChannel};
use stm32f4xx_hal::pac::SPI1;
use crate::devices::rtt_telemetry::RttTelemetry;

// global logger
pub struct MyBoard {
    button_pin: Option<Pin<'C', 13>>,
    cs_mot_a_pin: Option<Pin<'B', 6, Output>>,
    cs_mot_b_pin: Option<Pin<'C', 7, Output>>,
    cs_mot_c_pin: Option<Pin<'A', 9, Output>>,
    mot_a_lim_pin: Option<Pin<'B', 4>>,
    mot_b_lim_pin: Option<Pin<'B', 5>>,
    mot_c_lim_pin: Option<Pin<'A', 10>>,
    cs_touch_sense_pin: Option<Pin<'A', 8, Output>>,
    motor_enabler_pin: Option<Pin<'B', 7, Output>>,
    spi1: Option<Spi<stm32f4xx_hal::pac::SPI1>>,
    rtt_up_channels: [Option<UpChannel>;3],
    rtt_down_channels: [Option<DownChannel>;1],
}
pub struct InfallibleResources {
    pub motor_enabler: GPIOMotorEnabler<Pin<'B', 7, Output>>,
    pub log_device: RttLogger,
}
pub struct FallibleResources {
    pub button: GpioButton<Pin<'C', 13>>,
    pub stp_motor_drive_a:
        TMC5130StepperDev<Spidev<'static, Spi<SPI1>, Pin<'B', 6, Output>>, Pin<'B', 4>>,
    pub stp_motor_drive_b:
        TMC5130StepperDev<Spidev<'static, Spi<SPI1>, Pin<'C', 7, Output>>, Pin<'B', 5>>,
    pub stp_motor_drive_c:
        TMC5130StepperDev<Spidev<'static, Spi<SPI1>, Pin<'A', 9, Output>>, Pin<'A', 10>>,
}

static GUARDED_SPI: Mutex<RefCell<Option<Spi<SPI1>>>> = Mutex::new(RefCell::new(None));
impl MyBoard {
    pub fn new() -> Self {
        let dp = hal::pac::Peripherals::take().expect("cannot take peripherals");
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(150.MHz()).freeze();
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();
        let button_pin = Some(gpioc.pc13.into_pull_up_input());
        let mot_a_lim_pin = Some(gpiob.pb4.into_pull_up_input());
        let mot_b_lim_pin = Some(gpiob.pb5.into_pull_up_input());
        let mot_c_lim_pin = Some(gpioa.pa10.into_pull_up_input());
        let cs_mot_a_pin = gpiob.pb6.into_push_pull_output_in_state(PinState::High);
        let cs_mot_b_pin = gpioc.pc7.into_push_pull_output_in_state(PinState::High);
        let cs_mot_c_pin = gpioa.pa9.into_push_pull_output_in_state(PinState::High);
        let cs_touch_sense_pin = gpioa.pa8.into_push_pull_output_in_state(PinState::High);
        let motor_enabler_pin = gpiob.pb7.into_push_pull_output_in_state(PinState::Low);
        let spi = dp.SPI1.spi(
            (gpioa.pa5, gpioa.pa6, gpioa.pa7),
            hal::spi::Mode {
                polarity: Polarity::IdleLow,
                phase: Phase::CaptureOnFirstTransition,
            },
            2000.kHz(),
            &clocks,
        );
        let channels = rtt_init! {
            up: {
                0: {
                    size: 512,
                    mode: ChannelMode::NoBlockSkip,
                    name: "Menu output"
                }
                1: {
                    size: 1024,
                    mode: ChannelMode::NoBlockSkip,
                    name: "Log output"
                }
                2: {
                    size: 1024,
                    mode: ChannelMode::NoBlockSkip,
                    name: "Telemetry output"
                }
            }
            down: {
                0: {
                    size: 512,
                    mode: ChannelMode::NoBlockSkip,
                    name: "Menu Input"
                }
            }
        };
        MyBoard {
            button_pin,
            mot_a_lim_pin,
            mot_b_lim_pin,
            mot_c_lim_pin,
            motor_enabler_pin: Some(motor_enabler_pin),
            spi1: Some(spi),
            rtt_up_channels: [Some(channels.up.0), Some(channels.up.1), Some(channels.up.2)],
            cs_mot_a_pin: Some(cs_mot_a_pin),
            cs_mot_b_pin: Some(cs_mot_b_pin),
            cs_mot_c_pin: Some(cs_mot_c_pin),
            cs_touch_sense_pin: Some(cs_touch_sense_pin),
            rtt_down_channels: [Some(channels.down.0)],
        }
    }
}
impl BoardResources for MyBoard {
    type MotorEnabler = GPIOMotorEnabler<Pin<'B', 7, Output>>;
    type LogDevice = RttLogger;
    type Button = GpioButton<Pin<'C', 13>>;
    type StepperDriveA =
        TMC5130StepperDev<Spidev<'static, Spi<SPI1>, Pin<'B', 6, Output>>, Pin<'B', 4>>;
    type StepperDriveB =
        TMC5130StepperDev<Spidev<'static, Spi<SPI1>, Pin<'C', 7, Output>>, Pin<'B', 5>>;
    type StepperDriveC =
        TMC5130StepperDev<Spidev<'static, Spi<SPI1>, Pin<'A', 9, Output>>, Pin<'A', 10>>;
    type TouchSensor = Tsc2046TouchDev<Spidev<'static, Spi<SPI1>, Pin<'A', 8, Output>>>;
    type MenuIO = RttRWInterface;
    type TelemetrySender = RttTelemetry;

    fn get_infallible_resources(&mut self) -> (Self::MotorEnabler, Self::LogDevice, Self::MenuIO) {
        (
            GPIOMotorEnabler::new(self.motor_enabler_pin.take().unwrap()),
            RttLogger::new(self.rtt_up_channels[1].take().unwrap()),
            RttRWInterface::new(self.rtt_up_channels[0].take().unwrap(), self.rtt_down_channels[0].take().unwrap()),
        )
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
        let mut button = GpioButton::new(self.button_pin.take().unwrap());
        //let a: Result<(), Error> = spi.read();
        let spi = self.spi1.take().unwrap();
        interrupt::free(|cs| {
            GUARDED_SPI.borrow(cs).replace(Some(spi));
        });
        let mot_a_lim_pin = self.mot_a_lim_pin.take().unwrap();
        let mot_b_lim_pin = self.mot_b_lim_pin.take().unwrap();
        let mot_c_lim_pin = self.mot_c_lim_pin.take().unwrap();
        let mut cs_mot_a_pin = self.cs_mot_a_pin.take().unwrap();
        let mut cs_mot_b_pin = self.cs_mot_b_pin.take().unwrap();
        let mut cs_mot_c_pin = self.cs_mot_c_pin.take().unwrap();
        let mut cs_touch_sense_pin = self.cs_touch_sense_pin.take().unwrap();
        cs_mot_a_pin.set_high();
        cs_mot_b_pin.set_high();
        cs_mot_c_pin.set_high();
        cs_touch_sense_pin.set_high();
        let mot_a_driver_spi_device = Spidev::new(&GUARDED_SPI, cs_mot_a_pin);
        let mot_b_driver_spi_device = Spidev::new(&GUARDED_SPI, cs_mot_b_pin);
        let mot_c_driver_spi_device = Spidev::new(&GUARDED_SPI, cs_mot_c_pin);
        let touch_sense_driver_spi_device = Spidev::new(&GUARDED_SPI, cs_touch_sense_pin);
        let stp_motor_drive_a = TMC5130StepperDev::new(mot_a_driver_spi_device, mot_a_lim_pin)
            .map_err(|e| BoardCreationError::StepperDriveInitError(e, 0))?;
        let stp_motor_drive_b = TMC5130StepperDev::new(mot_b_driver_spi_device, mot_b_lim_pin)
            .map_err(|e| BoardCreationError::StepperDriveInitError(e, 1))?;
        let stp_motor_drive_c = TMC5130StepperDev::new(mot_c_driver_spi_device, mot_c_lim_pin)
            .map_err(|e| BoardCreationError::StepperDriveInitError(e, 2))?;
        let touch_sense_dev = Tsc2046TouchDev::new(touch_sense_driver_spi_device)
            .map_err(|e| BoardCreationError::TouchSensorInitError(e))?;
        let telemetry_sender = RttTelemetry::new(self.rtt_up_channels[2].take().unwrap());
        Ok((
            button,
            stp_motor_drive_a,
            stp_motor_drive_b,
            stp_motor_drive_c,
            touch_sense_dev,
            telemetry_sender,
        ))
    }
}

impl<SPI, PIN> From<ErrorWrapper<SpiDevError<SPI, PIN>>> for StepperDeviceError
where
    SPI: embedded_hal::spi::SpiBus,
    PIN: embedded_hal::digital::OutputPin,
{
    fn from(wrapper: ErrorWrapper<SpiDevError<SPI, PIN>>) -> StepperDeviceError {
        let comms_err = match wrapper.0 {
            SpiDevError::SPIError(spie) => match spie.kind() {
                ErrorKind::Overrun => CommsError::SPIOverrun,
                ErrorKind::ModeFault => CommsError::SPIModeFault,
                ErrorKind::FrameFormat => CommsError::SPIFrameFormat,
                ErrorKind::ChipSelectFault => CommsError::SPICSPIn,
                ErrorKind::Other => CommsError::Unknown,
                _ => CommsError::Unknown,
            },
            SpiDevError::CSPinError(_) => CommsError::SPICSPIn,
            SpiDevError::MutexError => CommsError::SPIMutex,
            SpiDevError::NotImplemented => CommsError::NotImplemented,
        };
        StepperDeviceError::CommunicationError(comms_err)
    }
}
impl<SPI, PIN> From<ErrorWrapper<SpiDevError<SPI, PIN>>> for CommsError
where
    SPI: embedded_hal::spi::SpiBus,
    PIN: embedded_hal::digital::OutputPin,
{
    fn from(value: ErrorWrapper<SpiDevError<SPI, PIN>>) -> Self {
        match value.0 {
            SpiDevError::SPIError(spie) => match spie.kind() {
                ErrorKind::Overrun => CommsError::SPIOverrun,
                ErrorKind::ModeFault => CommsError::SPIModeFault,
                ErrorKind::FrameFormat => CommsError::SPIFrameFormat,
                ErrorKind::ChipSelectFault => CommsError::SPICSPIn,
                ErrorKind::Other => CommsError::Unknown,
                _ => CommsError::Unknown,
            },
            SpiDevError::CSPinError(_) => CommsError::SPICSPIn,
            SpiDevError::MutexError => CommsError::SPIMutex,
            SpiDevError::NotImplemented => CommsError::NotImplemented,
        }
    }
}
impl Into<CommsError> for ErrorWrapper<stm32f4xx_hal::spi::Error> {
    fn into(self) -> CommsError {
        CommsError::NotImplemented
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
        Ok(())
    }
}
impl Button for Dummy {
    fn is_pressed(&mut self) -> bool {
        false
    }
}
impl Reader for Dummy {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        0
    }
}
impl core::fmt::Write for Dummy {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        Ok(())
    }
}
