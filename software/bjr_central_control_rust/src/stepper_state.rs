
use core::sync::atomic::{AtomicI32, Ordering};

pub struct StepperState {
    pub pos: AtomicI32,
    pub vel: f32,
}
impl StepperState {
    pub const fn new() ->StepperState {
        StepperState {
            pos: AtomicI32::new(0),
            vel: 0.0,
        }

    }
}
impl Default for StepperState {
    fn default() -> Self {
        StepperState {
            pos: AtomicI32::new(0),
            vel: 0.0,
        }
    }
}
