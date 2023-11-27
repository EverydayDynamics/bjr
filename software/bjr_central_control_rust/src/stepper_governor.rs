use crate::stepper_control::{StepperCtrlr, StepperCtrlTrait};
use crate::stepper_state::StepperState;

const BASE_FREQ: u32 = 33_000;
pub enum StackableStepperCtrlr <STPA, STPB, STPC>
where STPA: StepperCtrlTrait,
    STPB: StepperCtrlTrait,
    STPC: StepperCtrlTrait,
{
    StepperControllerA(STPA),
    StepperControllerB(STPB),
    StepperControllerC(STPC),
}

impl<STPA, STPB, STPC> StepperCtrlTrait for StackableStepperCtrlr<STPA, STPB, STPC>
    where STPA: StepperCtrlTrait,
          STPB: StepperCtrlTrait,
          STPC: StepperCtrlTrait,
{
    fn run_first_stage(&mut self) -> &StepperState {
        match self {
            StackableStepperCtrlr::StepperControllerA(ctrlr) => {ctrlr.run_first_stage()}
            StackableStepperCtrlr::StepperControllerB(ctrlr) => {ctrlr.run_first_stage()}
            StackableStepperCtrlr::StepperControllerC(ctrlr) => {ctrlr.run_first_stage()}
        }
    }

    fn run_second_stage(&mut self) {
        match self {
            StackableStepperCtrlr::StepperControllerA(ctrlr) => {ctrlr.run_second_stage()}
            StackableStepperCtrlr::StepperControllerB(ctrlr) => {ctrlr.run_second_stage()}
            StackableStepperCtrlr::StepperControllerC(ctrlr) => {ctrlr.run_second_stage()}
        }
    }

}
