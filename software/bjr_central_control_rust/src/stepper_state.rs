
use core::sync::atomic::{AtomicI32};

pub struct StepperState {
    pub pos: AtomicI32,
    pub vel: AtomicI32,
}
impl StepperState {
    pub const fn new() ->StepperState {
        StepperState {
            pos: AtomicI32::new(0),
            vel: AtomicI32::new(0),
        }

    }
}
impl Default for StepperState {
    fn default() -> Self {
        StepperState {
            pos: AtomicI32::new(0),
            vel: AtomicI32::new(0),
        }
    }
}
