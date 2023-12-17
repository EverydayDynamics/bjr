use core::fmt::Debug;
use dcmimu::DCMIMU;
use dcmimu::GRAVITY;
use embedded_hal::blocking::i2c::{Write, WriteRead};
use icm42670::{Address, Icm42670};
use icm42670::accelerometer::Accelerometer;
use icm42670::accelerometer::vector::F32x3;

pub struct EulerBase {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
pub type EulerAngles = EulerBase;
pub type EulerRates = EulerBase;
pub struct PlateAngleSensor<I2C>
{
    dcmimu: DCMIMU,
    last_micros: u64,
    imu: Icm42670<I2C>
}
impl<I2C, E> PlateAngleSensor<I2C>
    where I2C: Write<Error = E> + WriteRead<Error = E>,
          E: Debug,
{
    pub fn new(current_micros:u64, i2c_handle: I2C) -> Result<Self, icm42670::Error<E>>{
        let mut dcmimu = DCMIMU::new();
        let imu = Icm42670::new(i2c_handle, Address::Primary)?;
       Ok(PlateAngleSensor {
           dcmimu,
           last_micros: current_micros,
           imu,
       })
    }
    pub fn update(&mut self, micros: u64) -> Result<(EulerAngles, EulerRates), icm42670::accelerometer::Error<icm42670::Error<E>>>{
        const DEG_TO_RAD: f32 = core::f32::consts::PI / 180.0;
        let imu_gyros = self.imu.gyro_norm()?;
        let gyro_radps = F32x3{
            x: imu_gyros.x*DEG_TO_RAD,
            y: imu_gyros.y*DEG_TO_RAD,
            z: imu_gyros.z*DEG_TO_RAD,
        };
        let imu_accels = self.imu.accel_norm()?;
        let delta_micros = micros - self.last_micros;
        self.last_micros = micros;
        let (angles, biases) = self.dcmimu.update((
            gyro_radps.x,
            gyro_radps.y,
            gyro_radps.z), (
            imu_accels.x *GRAVITY,
            imu_accels.y *GRAVITY,
            imu_accels.z *GRAVITY,), delta_micros as f32 / 1000000.0 );
        let mut transformed_roll = angles.roll /DEG_TO_RAD;
        if transformed_roll < 0.0 {
            transformed_roll +=180.0;
        } else {
            transformed_roll -=180.0;
        }
        log::info!("angle  =  roll: {:+.04} pitch: {:+.04} yaw: {:+.04}\t\tGYRO  =  X: {:+.04} Y: {:+.04} Z: {:+.04}",
        transformed_roll, angles.pitch /DEG_TO_RAD, angles.yaw /DEG_TO_RAD, biases.x, biases.y, biases.z);
        Ok((EulerAngles{x: transformed_roll, y: angles.pitch /DEG_TO_RAD, z: angles.yaw /DEG_TO_RAD},
            EulerRates{
                x: gyro_radps.x - biases.x,
                y: gyro_radps.y - biases.y,
                z: gyro_radps.z - biases.z,
            }))
    }
}