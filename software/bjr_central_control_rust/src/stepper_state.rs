
pub const STEP_BASE_FREQ: u32 = 33_000;
pub const STEP_VELOCITY_SCALER: i32 = 1000;
pub const STEP_THRESHOLD: i32 = STEP_BASE_FREQ as i32 * STEP_VELOCITY_SCALER;
#[derive(Clone,Copy)]
pub struct StepperState {
    pub pos: i32,
    pub vel: i32,
}
impl Default for StepperState {
    fn default() -> Self {
        StepperState {
            pos: 0,
            vel: 0,
        }
    }
}
