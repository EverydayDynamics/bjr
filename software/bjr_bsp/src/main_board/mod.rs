use core::cell::RefCell;
use core::fmt::Arguments;
use bsp_traits::{Button, CommsError, Logger, MotorEnabler, MotorInput, MotorState, StepperDeviceError};
use bsp_traits::StepperMotorController;
use bsp_traits::TemperatureSensor;
use crate::boards::{BjrBoardResources, BjrBoardSupport, BoardCreationError};
use cortex_m::{interrupt, Peripherals};
use cortex_m::interrupt::Mutex;
use stm32f4xx_hal as hal;
use stm32f4xx_hal::gpio::{Output, Pin, gpiob::PB6, PinState};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::spi::{Phase, Polarity, Spi};

use panic_probe as _;
use defmt_rtt as _;
use embedded_hal::spi::{Error, ErrorKind};
use stm32f4xx_hal::gpio;
use stm32f4xx_hal::pac::SPI1;
use stm32f4xx_hal::rcc::Clocks;
use tmc5130::Tmc5130;
use crate::devices::{gpio_button::GpioButton, tmc5130_stepper_dev::TMC5130StepperDev};
use crate::devices::defmt_logger::DefmtLogger;
use crate::devices::gpio_motor_enabler::GPIOMotorEnabler;
use crate::utils::error_wrapper::ErrorWrapper;
use crate::utils::spidev::{Spidev, SpiDevError};

// global logger
pub struct MyBoard {
    button_pin: Option<Pin<'B', 5>>,
    cs_1_pin: Option<Pin<'B', 6, Output>>,
    motor_enabler_pin: Option<Pin<'B', 7, Output>>,
    spi1: Option<Spi<stm32f4xx_hal::pac::SPI1>>,
}
pub struct InfallibleResources {
    pub motor_enabler: GPIOMotorEnabler<Pin<'B', 7, Output>>,
    pub log_device: DefmtLogger,
}
pub struct FallibleResources {
    pub button: GpioButton<Pin<'B', 5>>,
    pub stp_motor_drive_a: TMC5130StepperDev<Spidev<'static, Spi<SPI1>, Pin<'B', 6, Output>>>,
    pub stp_motor_drive_b: Dummy,
    pub stp_motor_drive_c: Dummy,
}

static GUARDED_SPI: Mutex<RefCell<Option<Spi<SPI1>>>> =
    Mutex::new(RefCell::new(None));
impl MyBoard {
    pub fn new() -> Self {
        let dp = hal::pac::Peripherals::take().expect("cannot take peripherals");
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(150.MHz()).freeze();
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let button_pin = Some(gpiob.pb5.into_pull_up_input());
        let cs_pin = gpiob.pb6.into_push_pull_output_in_state(PinState::High);
        let motor_enabler_pin = gpiob.pb7.into_push_pull_output_in_state(PinState::Low);
        let spi = dp.SPI1.spi(
            (gpioa.pa5, gpioa.pa6, gpioa.pa7),
            hal::spi::Mode{ polarity: Polarity::IdleLow, phase: Phase::CaptureOnFirstTransition },
            1000.kHz(),
            &clocks,
        );
        MyBoard {
            button_pin,
            motor_enabler_pin: Some(motor_enabler_pin),
            spi1: Some(spi),
            cs_1_pin: Some(cs_pin)
        }
    }
    pub fn get_infallible_resources(&mut self) -> InfallibleResources {
        InfallibleResources{ motor_enabler: GPIOMotorEnabler::new(self.motor_enabler_pin.take().unwrap()), log_device: DefmtLogger {} }
    }
    pub fn get_fallible_resources(&mut self) -> Result<FallibleResources, BoardCreationError> {
        //let a: Result<(), Error> = spi.read();
        let spi = self.spi1.take().unwrap();
        interrupt::free(|cs| {
            GUARDED_SPI.borrow(cs).replace(Some(spi));
        });
        let cs_pin = self.cs_1_pin.take().unwrap();
        let driver_spi_device = Spidev::new(&GUARDED_SPI, cs_pin);
        let stp_motor_drive_a = TMC5130StepperDev::new(driver_spi_device).map_err(|e|BoardCreationError::StepperDriveInitError(e, 0))?;
        let button = GpioButton::new(self.button_pin.take().unwrap());
        Ok(FallibleResources {
            button: button,
            stp_motor_drive_a: stp_motor_drive_a,
            stp_motor_drive_b: Dummy {},
            stp_motor_drive_c: Dummy {},
        })
    }
}




impl<SPI, PIN> From<ErrorWrapper<SpiDevError<SPI, PIN>>> for StepperDeviceError
where
    SPI: embedded_hal::spi::SpiBus,
    PIN: embedded_hal::digital::OutputPin,
{
    fn from(wrapper: ErrorWrapper<SpiDevError<SPI, PIN>>) -> StepperDeviceError {
        let comms_err = match wrapper.0 {
            SpiDevError::SPIError(spie) => {
                match spie.kind() {
                    ErrorKind::Overrun => {CommsError::SPIOverrun}
                    ErrorKind::ModeFault => {CommsError::SPIModeFault}
                    ErrorKind::FrameFormat => {CommsError::SPIFrameFormat}
                    ErrorKind::ChipSelectFault => {CommsError::SPICSPIn}
                    ErrorKind::Other => {CommsError::Unknown}
                    _ => {CommsError::Unknown}
                }}
            SpiDevError::CSPinError(_) => {CommsError::SPICSPIn}
            SpiDevError::MutexError => {CommsError::SPIMutex}
            SpiDevError::NotImplemented => {CommsError::NotImplemented}
        };
        StepperDeviceError::CommunicationError(comms_err)
    }
}
impl Into<CommsError> for ErrorWrapper<stm32f4xx_hal::spi::Error>
{
    fn into(self) -> CommsError {
        CommsError::NotImplemented
    }
}
pub struct Dummy {}
impl MotorEnabler for Dummy {
    fn set_enable(&mut self, enable: bool) {
        todo!()
    }
}
impl StepperMotorController for Dummy {
    fn set_inputs(&mut self, inputs: MotorInput) -> Result<(), StepperDeviceError> {
        todo!()
    }

    fn get_state(&mut self) -> Result<MotorState, StepperDeviceError> {
        todo!()
    }
}
impl Button for Dummy {
    fn is_pressed(&mut self) -> bool {
        todo!()
    }
}
