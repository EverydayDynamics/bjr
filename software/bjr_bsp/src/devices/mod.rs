#[cfg(all(feature = "embedded", feature = "defmt"))]
pub mod defmt_logger;
#[cfg(feature = "embedded")]
pub mod gpio_button;
#[cfg(feature = "embedded")]
pub mod gpio_motor_enabler;
#[cfg(feature = "embedded")]
pub mod rtt_logger;
#[cfg(feature = "embedded")]
pub mod rtt_rw_interface;
#[cfg(feature = "embedded")]
pub mod tmc5130_stepper_dev;
#[cfg(feature = "embedded")]
pub mod tsc2046_touchscreen_dev;
#[cfg(feature = "embedded")]
pub mod rtt_telemetry;
