use crate::app::control_primitives::{KinState, PlateState};
use crate::app::parameter_manager::{KinBaseCircRad, KinMinHeight, PistonBodyLen};

use crate::app::parameter_manager::{parameter_manager, KinHingeOffset, KinJointCircRad};
use fast_math;
use libm::{cosf, sinf, sqrtf};

const SQ3: f32 = 1.732_050_8_f32;
fn aligned_state(
    angle_state: KinState,
    height_state: KinState,
    base_circle_radius: f32,
    hinge_offset: f32,
    joint_circle_radius: f32,
    min_height: f32,
    piston_body_len: f32,
) -> KinState {
    let px = base_circle_radius - joint_circle_radius * cosf(angle_state.pos);
    let py = height_state.pos + min_height + joint_circle_radius * sinf(angle_state.pos);
    let l = sqrtf(px * px + py * py - hinge_offset * hinge_offset);
    let beta = fast_math::atan2(py, px) - fast_math::atan2(hinge_offset, l);
    let sinb: f32 = sinf(beta);
    let sinbpa: f32 = sinf(beta + angle_state.pos);
    let v = height_state.speed * sinb + angle_state.speed * joint_circle_radius * sinbpa;
    let a = height_state.accel * sinb
        + angle_state.speed
            * angle_state.speed
            * joint_circle_radius
            * cosf(beta + angle_state.pos)
        + angle_state.accel * joint_circle_radius * sinbpa;
    KinState {
        pos: l - piston_body_len,
        speed: v,
        accel: a,
    }
}
pub fn inverse_kinematics(plate: &PlateState) -> [KinState; 3] {
    let base_circle_radius = parameter_manager().get::<KinBaseCircRad>();
    let hinge_offset = parameter_manager().get::<KinHingeOffset>();
    let joint_circle_radius = parameter_manager().get::<KinJointCircRad>();
    let min_height = parameter_manager().get::<KinMinHeight>();
    let pistion_body_len = parameter_manager().get::<PistonBodyLen>();
    [
        aligned_state(
            (plate.angle[0] * SQ3 - (plate.angle[1]*(-1.0))) / 2.0,
            plate.height,
            base_circle_radius,
            hinge_offset,
            joint_circle_radius,
            min_height,
            pistion_body_len,
        ),
        aligned_state(
            plate.angle[1]*(-1.0),
            plate.height,
            base_circle_radius,
            hinge_offset,
            joint_circle_radius,
            min_height,
            pistion_body_len,
        ),
        aligned_state(
            (-plate.angle[0] * SQ3 - (plate.angle[1]*(-1.0))) / 2.0,
            plate.height,
            base_circle_radius,
            hinge_offset,
            joint_circle_radius,
            min_height,
            pistion_body_len,
        ),
    ]
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    const EPSILON: f32 = 1e-6;
    #[test]
    fn test_level() {
        let test_level_plate = PlateState {
            height: KinState {
                pos: 0.0,
                speed: 0.0,
                accel: 0.0,
            },
            angle: [KinState::default(); 2],
        };
        let result = inverse_kinematics(&test_level_plate);
        assert!((result[0].pos).abs() < EPSILON);
        assert!((result[1].pos).abs() < EPSILON);
        assert!((result[2].pos).abs() < EPSILON);
    }
}
