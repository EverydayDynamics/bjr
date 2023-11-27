#![deny(unsafe_code)]
//#![deny(warnings)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

extern crate uom;

#[cfg(not(test))]
use panic_rtt_target as _panic_handler;

mod stepper_control;
mod stepper_state;
#[cfg(not(test))]
mod app;
mod stepper_governor;

#[cfg(test)]
mod mock_peripherals;
mod controller_task;
mod motor;
mod captive_linear_stepper;
mod actuator_num;
