use core::fmt::Arguments;
use crate::bsp::devices::mock_button::MockButton;
pub struct MyBoard {
}

impl MyBoard {
    pub fn new() -> Self {
        MyBoard {
        }
    }
}

impl BoardSupport for MyBoard {
    fn get_temperature_sensor(&self) -> &dyn TemperatureSensor {
        &self.temperature_sensor
    }

    fn get_stepper_motor_controller(&self) -> &dyn StepperMotorController {
    }
    fn get_button(&mut self) -> & mut dyn Button {
        &mut self.button
    }
}