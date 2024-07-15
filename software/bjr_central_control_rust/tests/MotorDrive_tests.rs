#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use bjr as _;
use stm32f4xx_hal as hal;
use stm32f4xx_hal::gpio::{Output, Pin, gpiob::PB6};
use stm32f4xx_hal::pac::SPI1;
use stm32f4xx_hal::spi::Spi;
use tmc5130::Tmc5130;
use bjr::utils::MotorDrive::MotorDrive;
use bjr::utils::spidev::{Spidev, SpiDevError};
static GUARDED_SPI: Mutex<RefCell<Option<Spi<SPI1>>>> =
    Mutex::new(RefCell::new(None));
struct State {
    test_driver: MotorDrive<Spidev<'static, Spi<SPI1>, PB6<Output>>>
}

#[defmt_test::tests]
mod tests {
    use core::cell::RefCell;
    use cortex_m::interrupt::Mutex;
    use cortex_m::interrupt;
    use defmt::{assert_eq, unwrap, assert};
    use defmt::export::acquire;
    use stm32f4xx_hal as hal;
    use hal::{gpio::NoPin, pac, prelude::*};
    use crate::{GUARDED_SPI, State};
    use hal::spi::{Polarity, Phase};
    use stm32f4xx_hal::gpio::PinState;
    use bjr::utils::spidev::Spidev;
    use bjr::utils::MotorDrive::MotorDrive;
    use bjr::utils::PlatformManager::{BJRMain, BJRPlatform};

    #[init]
    fn setup() -> super::State {
        defmt::info!("Initializing integration test: MotorDrive_tests...");
        let mut platform = BJRMain::initialize().unwrap();
        State { test_driver: MotorDrive::new(platform.take_motor_drive_spi().unwrap()) }
    }
    #[test]
    fn self_test(state: &mut super::State) {
        let self_test_result = state.test_driver.self_test().unwrap();
    }
}
