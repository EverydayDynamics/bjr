pub struct KinState{
    pos: f32,
    speed: f32,
    accel: f32,
}
pub struct PlateState {
    height: KinState,
    angle: [KinState;2],
}
pub struct PlateDelta {
    height: KinState,
    angle: [KinState;2],
}
pub struct ControlInputs{
    feed_forward: PlateState,
    ball_setpoint: [f32;2]

}

pub struct ControlOutputs{
    motor_velocities: [f32;2],
}
