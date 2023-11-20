#![deny(unsafe_code)]
//#![deny(warnings)]
#![cfg_attr(not(test), no_main)]
#![cfg_attr(not(test), no_std)]

#[cfg(not(test))]
use panic_halt as _;

mod stepper_control;
#[cfg(not(test))]
mod app;
mod stepper_governor;

#[cfg(test)]
mod mock_peripherals;
