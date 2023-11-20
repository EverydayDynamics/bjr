use crate::stepper_control::StepperCtrlr;
const BASE_FREQ: u32 = 33_000;

pub struct StepperGovernor <STP1, STP2, STP3>{
    stepper_controller_a: STP1,
    stepper_controller_b: STP2,
    stepper_controller_c: STP3,
}

