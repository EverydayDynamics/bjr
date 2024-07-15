mod temperature_sensor;
mod stepper_motor_controller;

use temperature_sensor::MyBoardTemperatureSensor;
use stepper_motor_controller::MyBoardStepperMotorController;
use super::traits::*;
use super::devices::gpio_button::GpioButton;
use super::devices::mock_button::MockButton;


pub struct MyBoard {
    temperature_sensor: MyBoardTemperatureSensor,
    stepper_motor_controller: MyBoardStepperMotorController,
    button: MockButton,
}

impl MyBoard {
    pub fn new() -> Self {
        MyBoard {
            temperature_sensor: MyBoardTemperatureSensor::new(),
            stepper_motor_controller: MyBoardStepperMotorController::new(),
            button: MockButton::new(),
        }
    }
}

impl BoardSupport for MyBoard {
    fn get_temperature_sensor(&self) -> &dyn TemperatureSensor {
        &self.temperature_sensor
    }

    fn get_stepper_motor_controller(&self) -> &dyn StepperMotorController {
        &self.stepper_motor_controller
    }
    fn get_button(&mut self) -> & mut dyn Button {
        &mut self.button
    }
}