use crate::app::control_primitives::{ControlInputs, Controller, KinState, PlateState};
use embedded_time::duration::Microseconds;
use embedded_time::fixed_point::FixedPoint;
use crate::app::parameter_manager::{MCAccel, MCKp, parameter_manager, PlatePDCtrlKd, PlatePDCtrlKp};
use libm::asinf;
use crate::utils::usec2sec;

#[derive(Default)]
pub struct PIDController {
    last_call_time: Microseconds<u64>,
    last_error: [f32;2],
}
impl Controller for PIDController {
    fn reset(&mut self, call_time: Microseconds<u64>) {
        self.last_error = [0.0,0.0];
        self.last_call_time = call_time;
    }

    fn update(&mut self, call_time: Microseconds<u64>, inputs: ControlInputs) -> PlateState {
        let g = 9.81f32;
        let k_p = parameter_manager().get::<PlatePDCtrlKp>();
        let k_d = parameter_manager().get::<PlatePDCtrlKd>();
        let mut output_angles = [KinState::default(); 2];
        for axis in 0..2 {
            let error = inputs.ball_setpoint[axis].pos - inputs.measured_ball_state[axis].pos;
            let error_d = (error-self.last_error[axis])/usec2sec((call_time- self.last_call_time).integer());
            self.last_error[axis] = error;
            let target_ball_accel = error* k_p + error_d * k_d;
            let target_angle = asinf((target_ball_accel/g)*(7.0/5.0));
            output_angles[axis].pos = target_angle;
        }
        self.last_call_time = call_time;

        PlateState {
            height: KinState{pos: 10e-3, speed: 0.0, accel:0.0},
            angle: output_angles,
        }
    }
}
