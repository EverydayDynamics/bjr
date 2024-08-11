use crate::app::control_primitives::{KinState, PlateState};

use fast_math;
use libm::{sinf, cosf, sqrtf};
const SQ3:f32 = 1.732050807568877293527446341505872367_f32;
const B:f32 = 40.0;
const C:f32 = 6.5;
const R:f32 = 17.5;
fn aligned_state(angle_state: KinState, height_state: KinState) -> KinState{
    let Px = B-R*cosf(angle_state.pos);
    let Py = height_state.pos+R*sinf(angle_state.pos);
    let l = sqrtf(Px*Px+Py*Py-C*C);
    let beta = fast_math::atan2(Py,Px)-fast_math::atan2(C,l);
    let sinb:f32 = sinf(beta);
    let sinbpa:f32 = sinf(beta+angle_state.pos);
    let v = height_state.speed*sinb+angle_state.speed*R*sinbpa;
    let a = height_state.accel*sinb+
        angle_state.speed*angle_state.speed*R*cosf(beta+angle_state.pos) +
        angle_state.accel*R*sinbpa;
    KinState{pos:l,speed:v, accel:a}
}
pub fn inverse_kinematics(plate: PlateState) -> [KinState;3]{
    [
        aligned_state((plate.angle[0]*SQ3-plate.angle[1])/2.0, plate.height),
        aligned_state(plate.angle[1], plate.height),
        aligned_state((-plate.angle[0]*SQ3-plate.angle[1])/2.0, plate.height),
    ]
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use heapless::spsc::Queue;
    const EPSILON:f32 = 0.1;
    #[test]
    fn test_level() {

        let test_level_plate = PlateState {
            height: KinState{
                pos: 130.0-18.464-10.0,
                speed: 0.0,
                accel: 0.0,
            },
            angle: [KinState::default();2],
        };
        let result = inverse_kinematics(test_level_plate);
        assert!((result[0].pos-103.746).abs() < EPSILON );
        assert!((result[1].pos-103.746).abs() < EPSILON );
        assert!((result[2].pos-103.746).abs() < EPSILON );

    }
}