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
use bjr::utils::spidev::{Spidev, SpiDevError};
static GUARDED_SPI: Mutex<RefCell<Option<Spi<SPI1>>>> =
    Mutex::new(RefCell::new(None));
struct State<'a> {
    test_driver: Tmc5130<Spidev<'a, Spi<SPI1>, PB6<Output>>>
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
    use bjr::utils::spidev::Spidev;

    #[init]
    fn setup() -> super::State<'static> {
        defmt::info!("Initializing integration test: TSC5130...");
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
            1000.kHz(),
            &clocks,
        );
        let cs_pin = gpiob.pb6.into_push_pull_output_in_state(PinState::High);
        interrupt::free(|cs| {
            GUARDED_SPI.borrow(cs).replace(Some(spi));
        });

        let driver_spi_device = Spidev::new(&GUARDED_SPI, cs_pin);
        defmt::info!("Initializing done.");
        State { test_driver: Tmc5130::new(driver_spi_device) }
    }
    #[test]
    fn set_chopconf(state: &mut super::State<'static>) {
        let mut chopconf = reg::CHOPCONF::default();
        chopconf.set_toff(3);
        chopconf.set_hstrt(4);
        chopconf.set_hend(1);
        chopconf.set_tbl(2);
        chopconf.set_vsense(true);
        chopconf.set_chm(false);
        let mut ihold_irun = reg::IHOLD_IRUN::default();
        ihold_irun.set_ihold(0);
        ihold_irun.set_irun(16);
        ihold_irun.set_ihold_delay(6);
        let mut tpowerdown = reg::TPOWERDOWN::from(10);
        let mut gconf = reg::GCONF::default();
        let mut tpwmthrs = reg::TPWMTHRS::from(500);
        let mut pwmconf = reg::PWMCONF::from(500);
        pwmconf.set_pwm_autoscale(false);
        pwmconf.set_pwm_ampl(200);
        pwmconf.set_pwm_grad(1);
        pwmconf.set_pwm_freq(0);
        gconf.set_en_pwm_mode(true);
        let mut a1 = reg::A1::from(1000);
        let mut v1 = reg::V1::from(50000);
        let mut amax = reg::AMAX::from(500);
        let mut vmax = reg::VMAX::from(200000);
        let mut dmax = reg::DMAX::from(700);
        let mut d1 = reg::D1::from(1400);
        let mut vstop = reg::VSTOP::from(10);
        let mut rampmode = reg::RAMPMODE::from(0);
        let mut xtarget = reg::XTARGET::default();
        xtarget.set(51200);
        //todo: start running with XTARTGET
        let mut vactual = reg::VACTUAL::default();
        let mut xtarget2 = reg::State::from(reg::XTARGET::default());

        let b_chopconf = tmc5130::reg::State::from(chopconf);
        let b_ihold_irun = tmc5130::reg::State::from(ihold_irun);
        let b_tpowerdown = tmc5130::reg::State::from(tpowerdown);
        let b_gconf = tmc5130::reg::State::from(gconf);
        let b_pwmconf = tmc5130::reg::State::from(pwmconf);
        let b_a1 = tmc5130::reg::State::from(a1);
        let b_v1 = tmc5130::reg::State::from(v1);
        let b_amax = tmc5130::reg::State::from(amax);
        let b_vmax = tmc5130::reg::State::from(vmax);
        let b_dmax = tmc5130::reg::State::from(dmax);
        let b_d1 = tmc5130::reg::State::from(d1);
        let b_vstop = tmc5130::reg::State::from(vstop);
        let b_rampmode = tmc5130::reg::State::from(rampmode);
        let b_xtarget = tmc5130::reg::State::from(xtarget);
        let mut actions = [
            tmc5130::Action::write(&b_chopconf),
            tmc5130::Action::write(&b_ihold_irun),
            tmc5130::Action::write(&b_tpowerdown),
            tmc5130::Action::write(&b_gconf),
            tmc5130::Action::write(&b_pwmconf),
            tmc5130::Action::write(&b_a1),
            tmc5130::Action::write(&b_v1),
            tmc5130::Action::write(&b_amax),
            tmc5130::Action::write(&b_vmax),
            tmc5130::Action::write(&b_dmax),
            tmc5130::Action::write(&b_d1),
            tmc5130::Action::write(&b_vstop),
            tmc5130::Action::write(&b_rampmode),
            tmc5130::Action::write(&b_xtarget),
            tmc5130::Action::read(&mut xtarget2),
        ];

        defmt::info!("Starting bulk action");
        let result = state.test_driver.bulk_register_action(&mut actions);
        defmt::info!("result: {}", result);
        let state_num: u32 = xtarget2.into();
        defmt::info!("xtarget data: {}", state_num);
    }
}