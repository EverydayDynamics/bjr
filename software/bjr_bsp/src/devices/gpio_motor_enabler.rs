use crate::devices::gpio_button::GpioButton;
use device_traits::MotorEnabler;
use embedded_hal::digital::OutputPin;

pub struct GPIOMotorEnabler<PIN> {
    pin: PIN,
}
impl<PIN> GPIOMotorEnabler<PIN>
where
    PIN: OutputPin,
{
    pub fn new(pin: PIN) -> Self {
        GPIOMotorEnabler { pin }
    }
}

impl<PIN> MotorEnabler for GPIOMotorEnabler<PIN>
where
    PIN: OutputPin,
{
    fn set_enable(&mut self, enable: bool) {
        if enable {
            self.pin.set_high().unwrap();
        } else {
            self.pin.set_low().unwrap();
        }
    }
}
