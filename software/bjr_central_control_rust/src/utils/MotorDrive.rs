use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use defmt::{Format, Formatter};
use embedded_hal::spi::SpiDevice;
use stm32f4xx_hal::gpio::{Output, PB6, Pin};
use stm32f4xx_hal::pac::SPI1;
use stm32f4xx_hal::spi::Spi;
use tmc5130::Tmc5130;
use crate::utils::MotorDrive::Error::TestErrorCase;
use crate::utils::spidev::Spidev;
#[derive(Debug, Format)]
pub enum Error {
    TestErrorCase,
}

pub struct MotorDrive<SPI> {
 drive_ic: Tmc5130<SPI>
}
impl<SPI> MotorDrive<SPI>
where SPI: SpiDevice<u8>,
{
    pub fn new(spi_dev: SPI ) -> Self {
        MotorDrive{drive_ic:Tmc5130::new(spi_dev)}
    }
    pub fn self_test(&self) -> Result<(), Error> {
        return Err(TestErrorCase)
    }
    pub fn set_velocity(target: i32) -> Result<i32, Error> {
        todo!();
    }
}
