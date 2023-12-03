
#[derive(Clone,Copy)]
pub struct MotorState {
    pub pos: f32, //mm
    pub vel: f32, //mm/s
    pub accel: f32, //mm/s^2
}
