use crate::app::control::control_primitives::PlateDelta;
use crate::app::event_handler::State;
use embedded_time::duration::*;
fn default(_time: Microseconds) -> PlateDelta {
    todo!()
}


pub trait FeedForwardGen{
    fn reset(&mut self, time: Microseconds<u64>);
    fn get_ff(&mut self, time: Microseconds<u64>) -> PlateDelta;
}

pub struct FFGenCH {
}
impl FeedForwardGen for FFGenCH{
    fn reset(&mut self, time: Microseconds<u64>) {
        todo!()
    }

    fn get_ff(&mut self, time: Microseconds<u64>) -> PlateDelta {
        todo!()
    }
}