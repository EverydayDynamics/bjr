use crate::plate_state::PlateState;
use crate::motor_state::MotorState;
use fast_math;
use libm::{sinf, cosf, sqrtf};
use num_traits::Float;
use core::f32::consts::PI;
const SQ3:f32 = 1.732050807568877293527446341505872367_f32;
const B:f32 = 40.0;
const C:f32 = 6.5;
const R:f32 = 17.5;
    fn aligned_state(angle:f32, angle_speed:f32, angle_accel:f32, height:f32, height_speed:f32, height_accel:f32) -> MotorState {
        let Px = B-R*angle.cos();
        let Py = height+R*angle.sin();
        let l = (Px*Px+Py*Py-C*C).sqrt();
        let beta = fast_math::atan2(Py,Px)-fast_math::atan2(C,l);
        let sinb:f32 = sinf(beta);
        let sinbpa:f32 = sinf(beta+angle);
        let v = height_speed*sinb+angle_speed*R*sinbpa;
        let a = height_accel*sinb+
                angle_speed*angle_speed*R*cosf(beta+angle) +
                angle_accel*R*sinbpa;
        MotorState{pos:l,vel:v, accel:a}
    }
    pub fn inverse(plate: PlateState) -> [MotorState;3]{
        [
            aligned_state(plate.angle_a, plate.angle_a_speed, plate.angle_a_accel, plate.height, plate.height_speed, plate.height_accel),
            aligned_state((-plate.angle_a+SQ3*plate.angle_b)/2.0, (-plate.angle_a_speed+SQ3*plate.angle_b_speed)/2.0, (-plate.angle_a_accel+SQ3*plate.angle_b_accel)/2.0, plate.height, plate.height_speed, plate.height_accel),
            aligned_state((-plate.angle_a-SQ3*plate.angle_b)/2.0, (-plate.angle_a_speed-SQ3*plate.angle_b_speed)/2.0, (-plate.angle_a_accel-SQ3*plate.angle_b_accel)/2.0, plate.height, plate.height_speed, plate.height_accel),
        ]
    }
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::mock_peripherals::MockOutput;
    use heapless::spsc::Queue;
    const EPSILON:f32 = 0.1;
    #[test]
    fn test_level() {
        let test_level_plate = PlateState {
            angle_a: 0.0,
            angle_b: 0.0,
            height: 130.0-18.464-10.0,
            angle_a_speed: 0.0,
            angle_b_speed: 0.0,
            height_speed: 10.0,
            angle_a_accel: 0.0,
            angle_b_accel: 0.0,
            height_accel: 0.0,
        };
        let result = inverse(test_level_plate);
        assert!((result[0].pos-103.746).abs() < EPSILON );
        assert!((result[1].pos-103.746).abs() < EPSILON );
        assert!((result[2].pos-103.746).abs() < EPSILON );

    }

    #[test]
    fn test_bad_add() {
    }
}
