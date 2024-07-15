use core::cell::RefCell;
use core::fmt::Debug;
use cortex_m::interrupt;
use cortex_m::interrupt::Mutex;
use defmt::Format;
use embedded_hal::spi::{Phase, Polarity, SpiDevice};
use rtic_monotonics::systick::fugit::RateExtU32;
use stm32f4xx_hal::gpio::{GpioExt, Output, PB5, PB6, PinState};
use stm32f4xx_hal::{hal, pac};
use stm32f4xx_hal::pac::SPI1;
use stm32f4xx_hal::prelude::{_stm32f4xx_hal_rcc_RccExt, _stm32f4xx_hal_spi_SpiExt, _stm32f4xx_hal_timer_TimerExt};
use stm32f4xx_hal::spi::Spi;
use crate::utils::MotorDrive::MotorDrive;
use crate::utils::spidev::Spidev;


#[derive(Debug, Format)]
pub enum Error {
    NotImplemented,
    ResourceTaken,
    PeripheralsTaken,
}
pub trait BJRPlatform {
    type Error: Debug + Format;

    fn take_motor_drive_spi(&mut self) -> Result <impl SpiDevice<u8>, Self::Error>;
    fn take_touchscreen_spi(&mut self) -> Result <impl SpiDevice<u8>, Self::Error>;
}

static GUARDED_SPI: Mutex<RefCell<Option<Spi<SPI1>>>> =
    Mutex::new(RefCell::new(None));
pub struct BJRMain {
    motor_drive_spi_a: Option<Spidev<'static, Spi<SPI1>, PB6<Output>>>,
    touchscreen_spi: Option<Spidev<'static, Spi<SPI1>, PB5<Output>>>,
}
impl BJRMain {
    pub fn initialize() -> Result<BJRMain, Error> {

        let dp = pac::Peripherals::take().ok_or(Error::PeripheralsTaken)?;
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(150.MHz()).freeze();
        let mut delay = dp.TIM1.delay_us(&clocks);
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let spi = dp.SPI1.spi(
            (gpioa.pa5, gpioa.pa6, gpioa.pa7),
            hal::spi::Mode{ polarity: Polarity::IdleLow, phase: Phase::CaptureOnFirstTransition },
            1000.kHz(),
            &clocks,
        );
        interrupt::free(|cs| {
            GUARDED_SPI.borrow(cs).replace(Some(spi));
        });
        let md_a_cs_pin = gpiob.pb6.into_push_pull_output_in_state(PinState::High);
        let touchscreen_cs_pin = gpiob.pb5.into_push_pull_output_in_state(PinState::High);
        let driver_spi_device = Spidev::new(&GUARDED_SPI, md_a_cs_pin);
        let touchscreen_spi_device = Spidev::new(&GUARDED_SPI, touchscreen_cs_pin);
        Ok(BJRMain{  motor_drive_spi_a: Some(driver_spi_device), touchscreen_spi:Some(touchscreen_spi_device) })
    }
}

impl BJRPlatform for BJRMain {
    type Error = Error;

    fn take_motor_drive_spi(&mut self) -> Result<Spidev<'static, Spi<SPI1>, PB6<Output>>, Self::Error> {
        return self.motor_drive_spi_a.take().ok_or(Error::ResourceTaken);
    }
    fn take_touchscreen_spi(&mut self) -> Result<Spidev<'static, Spi<SPI1>, PB5<Output>>, Self::Error> {
        return self.touchscreen_spi.take().ok_or(Error::ResourceTaken);
    }
}
