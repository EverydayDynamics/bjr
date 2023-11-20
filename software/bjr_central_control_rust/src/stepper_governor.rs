use crate::stepper_control::{StepperCtrlr, StepperCtrlTrait};
const BASE_FREQ: u32 = 33_000;
pub struct StepperGovernor <STP1, >
where STP1: StepperCtrlTrait,
{
    stepper_controller_a: STP1,
}

