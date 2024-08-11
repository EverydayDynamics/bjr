use embedded_time::duration::Microseconds;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
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
pub struct ControlExecInputs{
    pub measured_plate_angle: [KinState;2],
    pub measured_ball_state: [KinState;2],

}
pub struct ControlInputs{
    pub feed_forward: PlateDelta,
    pub ball_setpoint: [KinState;2],
    pub inputs: ControlExecInputs,
}

pub struct ControlOutputs{
    motor_velocities: [f32;3],
}
pub struct TelemetryPacket{

}
pub enum BallPattern{
    CenterHold,
    Triangle,
    Circling,
}
pub trait Controller {

    fn reset(&mut self, call_time: Microseconds<u64>);
    fn update(&mut self, call_time: Microseconds<u64>, _inputs: ControlInputs) -> ControlOutputs;
}