use core::fmt::{Display, Formatter};
use embedded_time::duration::Microseconds;
use embedded_time::fixed_point::FixedPoint;
use bjr_telemetry::{TelemetryData, TelemetryPacket};
use bsp_traits::{TelemetrySender, TelemetrySenderError};
use crate::app::control_primitives::KinState;
use postcard;

pub struct TelemetryHandler<TEL> {
    telemetry_sender: TEL,
    sent_packets: u64,
}
pub struct TelemetryBuilder{
    packet: TelemetryPacket,
}
impl TelemetryBuilder {
    pub fn add_ball_state_telemetry(&mut self, ball_state: &[KinState; 2]) {
        self.packet.set_field(TelemetryData::BallXPos(ball_state[0].pos));
        self.packet.set_field(TelemetryData::BallYPos(ball_state[1].pos));
        self.packet.set_field(TelemetryData::BallXVel(ball_state[0].speed));
        self.packet.set_field(TelemetryData::BallYVel(ball_state[1].speed));
    }
    pub fn add_unfiltered_ball_velocities(&mut self, unfiltered_ball_vels: &[f32; 2]) {
        self.packet.set_field(TelemetryData::UnfilteredBallXVel(unfiltered_ball_vels[0]));
        self.packet.set_field(TelemetryData::UnfilteredBallYVel(unfiltered_ball_vels[1]));
    }
    pub fn add_cpu_use(&mut self, cpu_use: f32) {
        self.packet.set_field(TelemetryData::CpuUse(cpu_use))
    }
    pub fn add_motor_target(&mut self, motor_idx:usize, motor_target: &KinState) {
        match motor_idx{
            0 => {
                self.packet.set_field(TelemetryData::MotorTargetAPos(motor_target.pos));
                self.packet.set_field(TelemetryData::MotorTargetAVel(motor_target.speed));
                self.packet.set_field(TelemetryData::MotorTargetAAccel(motor_target.accel));
            }
            1 => {
                self.packet.set_field(TelemetryData::MotorTargetBPos(motor_target.pos));
                self.packet.set_field(TelemetryData::MotorTargetBVel(motor_target.speed));
                self.packet.set_field(TelemetryData::MotorTargetBAccel(motor_target.accel));
            }
            2 => {
                self.packet.set_field(TelemetryData::MotorTargetCPos(motor_target.pos));
                self.packet.set_field(TelemetryData::MotorTargetCVel(motor_target.speed));
                self.packet.set_field(TelemetryData::MotorTargetCAccel(motor_target.accel));

            }
            _ => {}
        }

    }
    pub fn add_motor_state(&mut self, motor_idx:usize, motor_state: &KinState) {
        match motor_idx{
            0 => {
                self.packet.set_field(TelemetryData::MotorStateAPos(motor_state.pos));
                self.packet.set_field(TelemetryData::MotorStateAVel(motor_state.speed));
            }
            1 => {
                self.packet.set_field(TelemetryData::MotorStateBPos(motor_state.pos));
                self.packet.set_field(TelemetryData::MotorStateBVel(motor_state.speed));
            }
            2 => {
                self.packet.set_field(TelemetryData::MotorStateCPos(motor_state.pos));
                self.packet.set_field(TelemetryData::MotorStateCVel(motor_state.speed));

            }
            _ => {}
        }

    }
    pub fn add_motor_control(&mut self, motor_idx: usize, tracking_error: f32){
        match motor_idx {
            0 => {self.packet.set_field(TelemetryData::MotorACtrlTrackingError(tracking_error));}
            1 => {self.packet.set_field(TelemetryData::MotorBCtrlTrackingError(tracking_error));}
            2 => {self.packet.set_field(TelemetryData::MotorCCtrlTrackingError(tracking_error));}
            _ => {}
        }
    }
    pub fn get_packet(self) ->TelemetryPacket {
        self.packet
    }

}
pub enum TelemetryHandlerError {
    SendingError(TelemetrySenderError),
    SerializationError(postcard::Error),
}
impl Display for TelemetryHandlerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            TelemetryHandlerError::SendingError(e) => {write!(f,"Couldn't send telemetry: {}", e)}
            TelemetryHandlerError::SerializationError(_) => {write!(f,"Couldn't serialize telemetry")}
        }
    }
}
impl<TEL> TelemetryHandler<TEL>
where TEL: TelemetrySender
{
    pub fn new(telemetry_sender: TEL) -> TelemetryHandler<TEL> {
        TelemetryHandler{ telemetry_sender, sent_packets:0}
    }
    pub fn send_packet(&mut self, packet: TelemetryPacket) -> Result<(), TelemetryHandlerError> {
        let mut buffer = [0u8; 1024];
        let result = postcard::to_slice(&packet, &mut buffer).map_err(|e| TelemetryHandlerError::SerializationError(e))?;
        self.telemetry_sender.send(result).map_err(|e| TelemetryHandlerError::SendingError(e))
    }
    pub fn prepare_packet(&mut self, call_time: Microseconds<u64>) -> TelemetryBuilder{
        let ret = TelemetryBuilder{ packet: TelemetryPacket::new(call_time.integer(), self.sent_packets) };
        self.sent_packets +=1;
        ret
    }
}