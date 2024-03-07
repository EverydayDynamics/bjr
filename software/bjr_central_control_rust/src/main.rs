#![deny(unsafe_code)]
//#![deny(warnings)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

extern crate uom;

#[cfg(not(test))]
use panic_rtt_target as _;

mod stepper_state;
#[cfg(not(test))]
mod app;

#[cfg(test)]
mod mock_peripherals;

mod controller_task;
mod motor;
mod captive_linear_stepper;
mod actuator_num;
mod motor_controller;
mod stepper_driver;
mod stepper_controller2;
mod kinematics;
mod plate_state;
mod motor_state;
mod value_state;
mod initializer;
mod refcell_i2c_device;
mod plate_angle_sensor;
mod mutex_i2c_device;
mod TSC2046_driver;
