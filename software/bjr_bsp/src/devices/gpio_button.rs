use embedded_hal::digital::InputPin;
use bsp_traits::Button;
pub struct GpioButton<PIN> {
    pub(crate) pin: PIN,
}

impl<PIN> GpioButton<PIN>
    where
        PIN: InputPin,
{
    pub fn new(pin: PIN) -> Self {
        GpioButton { pin }
    }
}

impl<PIN> Button for GpioButton<PIN>
    where
        PIN: InputPin,
{
    fn is_pressed(&mut self) -> bool {
        // Assuming the button is connected in a pull-up configuration
        // where a low state means the button is pressed
        self.pin.is_low().unwrap_or(false)
    }
}