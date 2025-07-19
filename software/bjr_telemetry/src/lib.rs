#![cfg_attr(not(test), no_std)]
use serde::{Serialize, Deserialize};
use strum::{AsRefStr, EnumCount, VariantNames};

#[derive(Serialize, Deserialize)]
pub struct TelemetryPacket {
    pub timestamp: u64,
    pub packet_id: u64,
    pub data: [Option<TelemetryData>; TelemetryData::COUNT]
}
impl TelemetryPacket{
    pub fn new(timestamp:u64, packet_id:u64) -> TelemetryPacket {
        TelemetryPacket{
            timestamp,
            packet_id,
            data: [None;TelemetryData::COUNT]
        }
    }
}
#[derive(EnumCount, AsRefStr, Copy, Clone, Serialize, Deserialize, VariantNames)]
pub enum TelemetryData{
    BallXPos(f32),
    BallYPos(f32),
    BallXVel(f32),
    BallYVel(f32),
    MotorTargetAPos(f32),
    MotorTargetBPos(f32),
    MotorTargetCPos(f32),
    MotorTargetAVel(f32),
    MotorTargetBVel(f32),
    MotorTargetCVel(f32),
    MotorTargetAAccel(f32),
    MotorTargetBAccel(f32),
    MotorTargetCAccel(f32),
    MotorStateAPos(f32),
    MotorStateBPos(f32),
    MotorStateCPos(f32),
    MotorStateAVel(f32),
    MotorStateBVel(f32),
    MotorStateCVel(f32),
    UnfilteredBallXVel(f32),
    UnfilteredBallYVel(f32),
    CpuUse(f32),
    MotorACtrlTrackingError(f32),
    MotorBCtrlTrackingError(f32),
    MotorCCtrlTrackingError(f32),
    BallPressure(f32),
    BallXTargetPos(f32),
    BallYTargetPos(f32),
    BallXTargetVel(f32),
    BallYTargetVel(f32),
}
impl TelemetryData {
    pub fn get_idx(&self) -> usize{
        match self {
            TelemetryData::BallXPos(_) => 0,
            TelemetryData::BallYPos(_) => 1,
            TelemetryData::BallXVel(_) => 2,
            TelemetryData::BallYVel(_) => 3,
            TelemetryData::MotorTargetAPos(_) => 4,
            TelemetryData::MotorTargetBPos(_) => 5,
            TelemetryData::MotorTargetCPos(_) => 6,
            TelemetryData::MotorTargetAVel(_) => 7,
            TelemetryData::MotorTargetBVel(_) => 8,
            TelemetryData::MotorTargetCVel(_) => 9,
            TelemetryData::MotorTargetAAccel(_) => 10,
            TelemetryData::MotorTargetBAccel(_) => 11,
            TelemetryData::MotorTargetCAccel(_) => 12,
            TelemetryData::MotorStateAPos(_) => 13,
            TelemetryData::MotorStateBPos(_) => 14,
            TelemetryData::MotorStateCPos(_) => 15,
            TelemetryData::MotorStateAVel(_) => 16,
            TelemetryData::MotorStateBVel(_) => 17,
            TelemetryData::MotorStateCVel(_) => 18,
            TelemetryData::UnfilteredBallXVel(_) => 19,
            TelemetryData::UnfilteredBallYVel(_) => 20,
            TelemetryData::CpuUse(_) => 21,
            TelemetryData::MotorACtrlTrackingError(_) => 22,
            TelemetryData::MotorBCtrlTrackingError(_) => 23,
            TelemetryData::MotorCCtrlTrackingError(_) => 24,
            TelemetryData::BallPressure(_) => 25,
            TelemetryData::BallXTargetPos(_) => 26,
            TelemetryData::BallYTargetPos(_) => 27,
            TelemetryData::BallXTargetVel(_) => 28,
            TelemetryData::BallYTargetVel(_) => 29,
        }
    }
    pub fn get_printable_value(&self) -> f64 {
        let data  = match self {
            TelemetryData::BallXPos(data) => {data}
            TelemetryData::BallYPos(data) => {data}
            TelemetryData::BallXVel(data) => {data}
            TelemetryData::BallYVel(data) => {data}
            TelemetryData::MotorTargetAPos(data) => {data}
            TelemetryData::MotorTargetBPos(data) => {data}
            TelemetryData::MotorTargetCPos(data) => {data}
            TelemetryData::MotorTargetAVel(data) => {data}
            TelemetryData::MotorTargetBVel(data) => {data}
            TelemetryData::MotorTargetCVel(data) => {data}
            TelemetryData::MotorTargetAAccel(data) => {data}
            TelemetryData::MotorTargetBAccel(data) => {data}
            TelemetryData::MotorTargetCAccel(data) => {data}
            TelemetryData::MotorStateAPos(data) => {data}
            TelemetryData::MotorStateBPos(data) => {data}
            TelemetryData::MotorStateCPos(data) => {data}
            TelemetryData::MotorStateAVel(data) => {data}
            TelemetryData::MotorStateBVel(data) => {data}
            TelemetryData::MotorStateCVel(data) => {data}
            TelemetryData::UnfilteredBallXVel(data) => data,
            TelemetryData::UnfilteredBallYVel(data) => data,
            TelemetryData::CpuUse(data) => data,
            TelemetryData::MotorACtrlTrackingError(data) => data,
            TelemetryData::MotorBCtrlTrackingError(data) => data,
            TelemetryData::MotorCCtrlTrackingError(data) => data,
            TelemetryData::BallPressure(data) => data,
            TelemetryData::BallXTargetPos(data) => data,
            TelemetryData::BallYTargetPos(data) => data,
            TelemetryData::BallXTargetVel(data) => data,
            TelemetryData::BallYTargetVel(data) => data,
        };
        *data as f64
    }
}
impl TelemetryPacket{
    pub fn set_field(&mut self, data: TelemetryData){
        let idx = data.get_idx();
        self.data[idx] = Some(data);
    }
}