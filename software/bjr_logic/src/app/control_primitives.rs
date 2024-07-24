#[derive(Default)]
pub struct KinState{
    pub pos: f32,
    pub speed: f32,
    pub accel: f32,
}
pub struct PlateState {
    height: KinState,
    angle: [KinState;2],
}

#[derive(Default)]
pub struct PlateDelta {
    height: KinState,
    angle: [KinState;2],
}
pub struct ControlInputs{
    pub measured_plate_angle: [KinState;2],
    pub feed_forward: PlateDelta,
    pub ball_setpoint: [KinState;2],
    pub measured_ball_state: [KinState;2],
}

pub struct ControlOutputs{
    motor_velocities: [f32;2],
}
pub struct TelemetryPacket{

}