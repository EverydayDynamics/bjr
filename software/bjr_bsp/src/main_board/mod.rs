use core::cell::RefCell;
use core::fmt::Arguments;
use bsp_traits::{Button, CommsError, Logger, MotorEnabler, MotorInput, MotorState, StepperDeviceError};
use bsp_traits::StepperMotorController;
use bsp_traits::TemperatureSensor;
use crate::boards::{BjrBoardResources, BjrBoardSupport, BoardCreationError};
use cortex_m::interrupt;
use cortex_m::interrupt::Mutex;
use stm32f4xx_hal as hal;
use stm32f4xx_hal::gpio::{Output, Pin, gpiob::PB6, PinState};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::spi::{Phase, Polarity, Spi};

use panic_probe as _;
use defmt_rtt as _;
use embedded_hal::spi::{Error, ErrorKind};
use stm32f4xx_hal::pac::SPI1;
use tmc5130::Tmc5130;
use crate::devices::{gpio_button::GpioButton, tmc5130_stepper_dev::TMC5130StepperDev};
use crate::utils::error_wrapper::ErrorWrapper;
use crate::utils::spidev::{Spidev, SpiDevError};

// global logger
pub struct MyBoard {
    pub button: GpioButton<Pin<'B', 5>>,
    pub stp_motor_drive_a: TMC5130StepperDev<Spidev<'static, Spi<SPI1>, Pin<'B', 6, Output>>>,
    pub stp_motor_drive_b: Dummy,
    pub stp_motor_drive_c: Dummy,
    pub motor_enabler: Dummy,
    pub log_device: Dummy,

}

static GUARDED_SPI: Mutex<RefCell<Option<Spi<SPI1>>>> =
    Mutex::new(RefCell::new(None));
impl MyBoard {
    pub fn new() -> Result<Self, BoardCreationError> {
        let dp = hal::pac::Peripherals::take().expect("cannot take peripherals");
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(150.MHz()).freeze();

        let mut delay = dp.TIM1.delay_us(&clocks);
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let button_pin = gpiob.pb5.into_pull_up_input();
        let spi = dp.SPI1.spi(
            (gpioa.pa5, gpioa.pa6, gpioa.pa7),
            hal::spi::Mode{ polarity: Polarity::IdleLow, phase: Phase::CaptureOnFirstTransition },
            1000.kHz(),
            &clocks,
        );
        //let a: Result<(), Error> = spi.read();
        let cs_pin = gpiob.pb6.into_push_pull_output_in_state(PinState::High);
        interrupt::free(|cs| {
            GUARDED_SPI.borrow(cs).replace(Some(spi));
        });

        let driver_spi_device = Spidev::new(&GUARDED_SPI, cs_pin);
        let stp_motor_drive_a = TMC5130StepperDev::new(driver_spi_device).map_err(|e|BoardCreationError::StepperDriveInitError(e, 0))?;
        let button = GpioButton::new(button_pin);
        Ok(MyBoard {
            button: button,
            stp_motor_drive_a,
            log_device: Dummy{},
            stp_motor_drive_b: Dummy{},
            stp_motor_drive_c: Dummy{},
            motor_enabler: Dummy{},
        })
    }
}
impl BjrBoardSupport for MyBoard {
    fn get_stepper_motor_controller_a(&mut self) -> impl StepperMotorController {
        Dummy{}
    }

    fn get_stepper_motor_controller_b(&mut self) -> impl StepperMotorController {
        Dummy{}
    }

    fn get_stepper_motor_controller_c(&mut self) -> impl StepperMotorController {
        Dummy{}
    }

    fn get_button(& self) -> impl Button {
        Dummy{}
    }

    fn get_motor_enabler(&mut self) -> impl MotorEnabler {
        Dummy{}
    }

    fn get_log_device(&mut self) -> impl Logger {
        Dummy{}
    }

    fn get_resources(&mut self) -> BjrBoardResources {
        BjrBoardResources{
            stepper_devices: Some([&mut self.stp_motor_drive_a,&mut self.stp_motor_drive_b,&mut self.stp_motor_drive_c]),
            button: Some(&mut self.button),
            motor_enabler: Some(&mut self.motor_enabler),
            log_device: Some(&mut self.log_device),
        }
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
impl Logger for Dummy {
    fn trace(&self, args: Arguments<'_>) {
        todo!()
    }

    fn debug(&self, args: Arguments<'_>) {
        todo!()
    }

    fn info(&self, args: Arguments<'_>) {
        todo!()
    }

    fn warn(&self, args: Arguments<'_>) {
        todo!()
    }

    fn error(&self, args: Arguments<'_>) {
        todo!()
    }
}
impl Button for Dummy {
    fn is_pressed(&mut self) -> bool {
        todo!()
    }
}
