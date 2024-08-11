use crate::app::control_primitives::{KinState, PlateDelta};
use crate::app::event_handler::State;
use embedded_time::duration::*;
pub trait FeedForwardGen{
    fn reset(&mut self, time: Microseconds<u64>);
    fn get_ff(&mut self, time: Microseconds<u64>) -> PlateDelta;
}

pub struct FFGenCH {
}
impl FeedForwardGen for FFGenCH{
    fn reset(&mut self, time: Microseconds<u64>) {
    }

    fn get_ff(&mut self, time: Microseconds<u64>) -> PlateDelta {
        PlateDelta{ height: Default::default(), angle: [KinState::default();2] }

    }
}