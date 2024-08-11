use embedded_time::duration::Microseconds;
use crate::app::control_primitives::{ControlExecInputs, ControlInputs, Controller};
use crate::app::control::feedforward_generator::FeedForwardGen;
use crate::app::control::setpoint_generator::SetPointGen;
pub mod feedforward_generator;
pub mod pid_controller;
pub mod inverse_kinematics;
pub mod setpoint_generator;
