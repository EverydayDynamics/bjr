use embedded_time::duration::Microseconds;
use crate::app::control::control_primitives::PlateDelta;

pub trait SetPointGen{
    fn reset(&mut self, time: Microseconds<u64>);
    fn get_sp(&mut self, time: Microseconds<u64>) -> PlateDelta;
}
