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
use tsc2046::Tsc2046;
use bjr::utils::spidev::{Spidev, SpiDevError};
static GUARDED_SPI: Mutex<RefCell<Option<Spi<SPI1>>>> =
    Mutex::new(RefCell::new(None));
struct State<'a> {
    test_driver: Tsc2046<Spidev<'a, Spi<SPI1>, PB6<Output>>>
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
    use tmc5130::{reg, Tmc5130};
    use tmc5130::reg::Address::CHOPCONF;
    use tmc5130::reg::XACTUAL;
    use bjr::utils::spidev::{Spidev, SpiDevError};

    #[init]
    fn setup() -> super::State<'static> {
        defmt::info!("Initializing integration test: TSC2046...");
        let dp = pac::Peripherals::take().expect("cannot take peripherals");

        // Configure APB bus clock to 48 MHz, cause ws2812b requires 3 Mbps SPI
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.sysclk(150.MHz()).freeze();

        let mut delay = dp.TIM1.delay_us(&clocks);
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let spi = dp.SPI1.spi(
            (gpioa.pa5, gpioa.pa6, gpioa.pa7),
            hal::spi::Mode{ polarity: Polarity::IdleLow, phase: Phase::CaptureOnFirstTransition },
            100.kHz(),
            &clocks,
        );
        let cs_pin = gpiob.pb6.into_push_pull_output_in_state(PinState::High);
        interrupt::free(|cs| {
            GUARDED_SPI.borrow(cs).replace(Some(spi));
        });

        let driver_spi_device = Spidev::new(&GUARDED_SPI, cs_pin);
        defmt::info!("Initializing done.");
        State { test_driver: tsc2046::Tsc2046::new(driver_spi_device, false, 100.0).expect("Could not create driver")}
    }
    #[test]
    fn test_touch(state: &mut super::State<'static>) {
        loop {
            let result =state.test_driver.get_touch();
            match result {
                Ok(maybe_touch) => {
                    if let Some(touch) =maybe_touch{
                        defmt::info!("touch detected X:{}, Y:{}, Z:{}",touch.x, touch.y,touch.z)
                    } else {
                        defmt::info!("No touch detected");
                    }}
                Err(error) => {
                    defmt::error!{"{}",error}}
            };

        }
    }
}
