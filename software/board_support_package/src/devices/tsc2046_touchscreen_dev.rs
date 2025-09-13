use crate::utils::error_wrapper::ErrorWrapper;
use device_traits::{CommsError, TouchPoint, TouchSensor, TouchSensorError};
use embedded_hal::spi::SpiDevice;
use tsc2046::Tsc2046;

pub struct Tsc2046TouchDev<SPI> {
    driver: Tsc2046<SPI>,
}
impl<SPI> Tsc2046TouchDev<SPI>
where
    SPI: SpiDevice,
    CommsError: From<ErrorWrapper<<SPI as embedded_hal::spi::ErrorType>::Error>>,
{
    pub fn new(spi_dev: SPI) -> Result<Self, TouchSensorError> {
        Ok(Tsc2046TouchDev {
            driver: Tsc2046::new(spi_dev, true, 5.0f32)
                .map_err(|e| TouchSensorError::CommunicationError(ErrorWrapper(e).into()))?,
        })
    }
}
impl<SPI> TouchSensor for Tsc2046TouchDev<SPI>
where
    SPI: SpiDevice,
    CommsError: From<ErrorWrapper<<SPI as embedded_hal::spi::ErrorType>::Error>>,
{
    fn get_touch(&mut self) -> Result<Option<TouchPoint>, TouchSensorError> {
        let maybe_touch = self
            .driver
            .get_touch()
            .map_err(|e| TouchSensorError::CommunicationError(ErrorWrapper(e).into()))?;
        if let Some(touch) = maybe_touch {
            Ok(Some(TouchPoint {
                x: touch.x as i32,
                y: touch.y as i32,
                pressure: touch.z,
            }))
        } else {
            Ok(None)
        }
    }
}
