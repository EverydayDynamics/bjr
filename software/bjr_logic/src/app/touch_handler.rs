use crate::app::control_primitives::KinState;
use crate::app::parameter_manager::{LogicRunnerPeriodUs, parameter_manager, TouchCenterOffsetX, TouchCenterOffsetY, TouchScaleXX, TouchScaleXY, TouchScaleYX, TouchScaleYY};
use crate::utils::usec2sec;
use device_traits::{TouchSensor, TouchSensorError};
use crate::app::telemetry_handler::TelemetryBuilder;
use heapless::Deque;
use nalgebra::SMatrix;
use libm::powf;

pub struct TouchHandler<TS, DIFF> {
    touch_sensor: TS,
    differentiators: [DIFF;2],
    unfiltered_differentiators: [TrivialDiff;2],
}
impl<TS, DIFF> TouchHandler<TS, DIFF>
where
    TS: TouchSensor,
    DIFF: Differentiator,
{
    pub fn new(
        touch_sensor: TS,
        differentiators: [DIFF;2],
    ) -> TouchHandler<TS, DIFF> {
        TouchHandler {
            touch_sensor,
            differentiators,
            unfiltered_differentiators: [TrivialDiff::new(),TrivialDiff::new()],
        }
    }
    pub fn get_ball_state(
        &mut self,
        telemetry_builder: &mut TelemetryBuilder
    ) -> Result<Option<[KinState; 2]>, TouchSensorError> {
        let mut result = Ok(None);
        if let Some(touch_pos) = self.touch_sensor.get_touch()? {

            let delta_t = usec2sec(parameter_manager().get::<LogicRunnerPeriodUs>()as u64);
            let x_offset = parameter_manager().get::<TouchCenterOffsetX>();
            let y_offset = parameter_manager().get::<TouchCenterOffsetY>();
            let xx_scale = parameter_manager().get::<TouchScaleXX>();
            let xy_scale = parameter_manager().get::<TouchScaleXY>();
            let yx_scale = parameter_manager().get::<TouchScaleYX>();
            let yy_scale = parameter_manager().get::<TouchScaleYY>();
            let x_offseted = (touch_pos.x + x_offset)as f32;
            let y_offseted = (touch_pos.y + y_offset)as f32;
            let x_pos = x_offseted * xx_scale + y_offseted * xy_scale;
            let y_pos = x_offseted * yx_scale + y_offseted * yy_scale;
            self.differentiators[0].add_sample(x_pos);
            self.differentiators[1].add_sample(y_pos);
            self.unfiltered_differentiators[0].add_sample(x_pos);
            self.unfiltered_differentiators[1].add_sample(y_pos);
            let state = [KinState{
                pos: x_pos,
                speed: self.differentiators[0].get_diff(delta_t).unwrap_or(0.0),
                accel: 0.0,
            },KinState{
                pos: y_pos,
                speed:self.differentiators[1].get_diff(delta_t).unwrap_or(0.0),
                accel: 0.0,
            }];
            telemetry_builder.add_ball_state_telemetry(&state);
            telemetry_builder.add_unfiltered_ball_velocities(
                &[self.unfiltered_differentiators[0].get_diff(delta_t).unwrap_or(0.0),
                self.unfiltered_differentiators[0].get_diff(delta_t).unwrap_or(0.0)]
            );
            result = Ok(Some(state));
        } else {
           for differentiator in &mut self.differentiators{
               differentiator.reset();
           }
            for unfiltered_diff in &mut self.unfiltered_differentiators{
                unfiltered_diff.reset();
            }
        }
        result
    }
}
pub trait Differentiator {
    fn reset(&mut self);
    fn add_sample(&mut self, data: f32);
    fn get_diff(&mut self, delta_t: f32) -> Option<f32>;
}
//Taylor series matrix with 5 members:
// [ (-4)^0 (-3)^0 (-2)^0 (-1)^0 (0)^0]
// [ (-4)^1 (-3)^1 (-2)^1 (-1)^1 (0)^1]
// [ (-4)^2 (-3)^2 (-2)^2 (-1)^2 (0)^2]
// [ (-4)^3 (-3)^3 (-2)^3 (-1)^3 (0)^3]
// [ (-4)^4 (-3)^4 (-2)^4 (-1)^4 (0)^4]
// the inverse is:
//[
//[ 0.        ,  0.25      ,  0.45833333,  0.25      ,  0.04166667],
//[-0.        , -1.33333333, -2.33333333, -1.16666667, -0.16666667],
//[-0.        ,  3.        ,  4.75      ,  2.        ,  0.25      ],
//[ 0.        , -4.        , -4.33333333, -1.5       , -0.16666667],
//[ 1.        ,  2.08333333,  1.45833333,  0.41666667,  0.04166667]]
//for the first derivative we need the second column
pub struct SGDifferentiator<const WIN: usize, const ORD:usize> {
    ring_buffer: Deque<f32,21>,
    coeffs:[f32;21]
}
impl<const WIN: usize, const ORD:usize> Default for SGDifferentiator<WIN, ORD> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WIN: usize, const ORD:usize> SGDifferentiator<WIN, ORD> {
    pub fn new()->SGDifferentiator<WIN, ORD> {
        let mut vandermondre:SMatrix<f32, WIN,ORD> = SMatrix::zeros();
        for i in 0..WIN {
            for j in 0..ORD {
                vandermondre[(i, j)] = powf(i as f32-(WIN/2) as f32,j as f32);
            }
        }
        SGDifferentiator{
            ring_buffer: Deque::new(), coeffs:
            [
                -0.012_987_013 ,
                -0.011_688_312 ,
                -0.010_389_61 ,
                -0.009_090_909 ,
                -0.007_792_208 ,
                -0.006_493_506_5 ,
                -0.005_194_805 ,
                -0.003_896_104 ,
                -0.002_597_402_5 ,
                -0.001_298_701_3 ,
                0.0 ,
                0.001_298_701_3 ,
                0.002_597_402_5 ,
                0.003_896_104 ,
                0.005_194_805 ,
                0.006_493_506_5 ,
                0.007_792_208 ,
                0.009_090_909 ,
                0.010_389_61 ,
                0.011_688_312 ,
                0.012_987_013 ,
            ]
        }
    }
}
impl<const WIN: usize, const ORD:usize> Differentiator for SGDifferentiator<WIN, ORD> {
    fn reset(&mut self) {
        self.ring_buffer.clear();
    }

    fn add_sample(&mut self, data: f32) {
        if self.ring_buffer.is_full() {
            let _ = self.ring_buffer.pop_front();
        }
        self.ring_buffer.push_back(data).unwrap();
    }

    fn get_diff(&mut self, delta_t:f32) -> Option<f32> {
        //const SERIES: [f32;5] = [
        //     0.25      ,
        //    -1.33333333,
        //     3.0        ,
        //    -4.0        ,
        //     2.08333333,
        //];
        if self.ring_buffer.is_full() {
            let mut diff = 0.0f32;
            for(sample,coeff) in self.ring_buffer.iter().zip(self.coeffs) {
                diff += coeff*sample;
            }
            diff /= delta_t;
            Some(diff)
        } else{
            None
        }
    }
}
pub struct TrivialDiff {
    ring_buffer: Deque<f32,2>,
}
impl Default for TrivialDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl TrivialDiff{
    pub fn new()->TrivialDiff {
        TrivialDiff{
            ring_buffer: Deque::new(),
        }
    }
}
impl Differentiator for TrivialDiff {
    fn reset(&mut self) {
        self.ring_buffer.clear();
    }

    fn add_sample(&mut self, data: f32) {
        if self.ring_buffer.is_full() {
            let _ = self.ring_buffer.pop_front();
        }
        self.ring_buffer.push_back(data).unwrap();
    }

    fn get_diff(&mut self, delta_t:f32) -> Option<f32> {
        const SERIES: [f32;2] = [-1.0, 1.0];
        if self.ring_buffer.is_full() {
            let mut diff = 0.0f32;
            for(sample,coeff) in self.ring_buffer.iter().zip(SERIES) {
                diff += coeff*sample;
            }
            diff /= delta_t;
            Some(diff)
        } else{
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::app::control_primitives::KinState;
    use crate::app::parameter_manager::{
    };
    use crate::app::touch_handler::{Differentiator, Taylor5Differentiator, TouchHandler};
    use device_traits::Point;
    use embedded_time::duration::{Microseconds, Seconds};
    #[test]
    fn test_differentiator() {
        let delta_t = 0.001f32;
        let mut test_differentiator = Taylor5Differentiator::new();
        for sample in 0..5{
            assert!(test_differentiator.get_diff(delta_t).is_none());
            test_differentiator.add_sample(1.0);
        }
        assert!(test_differentiator.get_diff(delta_t).unwrap() < 1e-6);

        for sample in 0..5{
            test_differentiator.add_sample((sample as f32)*delta_t);
        }
        assert!((test_differentiator.get_diff(delta_t).unwrap()-1.0) < 1e-6);


    }
}
