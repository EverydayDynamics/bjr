use crate::app::control_primitives::KinState;
use crate::app::parameter_manager::{parameter_manager, TouchCenterOffsetX, TouchCenterOffsetY, TouchScaleXX, TouchScaleXY, TouchScaleYX, TouchScaleYY};
use crate::utils::usec2sec;
use bsp_traits::{TouchSensor, TouchSensorError};
use embedded_time::duration::Microseconds;
use embedded_time::fixed_point::FixedPoint;

pub struct TouchHandler<TS> {
    touch_sensor: TS,
    last_state: Option<[KinState; 2]>,
    last_call_time: Microseconds<u64>,
}
impl<TS> TouchHandler<TS>
where
    TS: TouchSensor,
{
    pub fn new(
        touch_sensor: TS,
        initial_call_time: Microseconds<u64>,
        initial_state: Option<[KinState; 2]>,
    ) -> TouchHandler<TS> {
        TouchHandler {
            touch_sensor,
            last_state: initial_state,
            last_call_time: initial_call_time,
        }
    }
    fn calc_kin(
        &mut self,
        pos: f32,
        call_time: Microseconds<u64>,
        last_state: KinState,
    ) -> KinState {
        let delta_t = usec2sec((call_time - self.last_call_time).integer());
        let speed = (pos - last_state.pos) / delta_t;
        let accel = (speed - last_state.speed) / delta_t;
        let state: KinState = KinState { pos: (pos+last_state.pos)/2.0, speed, accel };

        state
    }
    pub fn get_ball_state(
        &mut self,
        call_time: Microseconds<u64>,
    ) -> Result<Option<[KinState; 2]>, TouchSensorError> {
        let mut result = Ok(None);
        if let Some(touch_pos) = self.touch_sensor.get_touch()? {
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
            let mut new_sate = None;
            if let Some(last_state) = self.last_state {
                let xkinstate = self.calc_kin(x_pos, call_time, last_state[0]);
                let ykinstate = self.calc_kin(y_pos, call_time, last_state[1]);
                new_sate = Some([xkinstate, ykinstate]);
                result = Ok(new_sate);
            } else {
                let x_trivial_state = KinState {
                    pos: x_pos,
                    speed: 0.0,
                    accel: 0.0,
                };
                let y_trivial_state = KinState {
                    pos: y_pos,
                    speed: 0.0,
                    accel: 0.0,
                };
                new_sate = Some([x_trivial_state, y_trivial_state]);
                result = Ok(new_sate);
            }
            self.last_state = new_sate;
            self.last_call_time = call_time;
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use crate::app::control_primitives::KinState;
    use crate::app::parameter_manager::{
        parameter_manager, TouchCenterOffsetX, TouchCenterOffsetY, TouchScaleX, TouchScaleY,
    };
    use crate::app::touch_handler::TouchHandler;
    use crate::utils::test_helper::bsp_mocks::MockTouchSensor;
    use bsp_traits::Point;
    use embedded_time::duration::{Microseconds, Seconds};
    fn test_recover_from_lost_ball() {
        let expected_touch_x = (0.01 / parameter_manager().get::<TouchScaleX>()) as i32
            - parameter_manager().get::<TouchCenterOffsetX>();
        let expected_touch_y = (0.01 / parameter_manager().get::<TouchScaleY>()) as i32
            - parameter_manager().get::<TouchCenterOffsetY>();
        let mut mock_touch_sensor = MockTouchSensor::new();
        mock_touch_sensor
            .expect_get_touch()
            .times(1)
            .returning(move || {
                Ok(Some(Point {
                    x: expected_touch_x,
                    y: expected_touch_y,
                }))
            });
        let mut test_touch_handler =
            TouchHandler::new(mock_touch_sensor, Microseconds::<u64>::new(0), None);
        let result = test_touch_handler.get_ball_state(Seconds::new(1).into());
        //let expected_result = Ok(Some([KinState{
        //    pos: 0.1,
        //    speed: 0.0,
        //    accel: 0.0,
        //},KinState{
        //    pos: 0.1,
        //    speed: 0.0,
        //    accel: 0.0,
        //}]));
        //assert!(expected_result == result);
    }
    fn test_lost_ball() {
        let mut mock_touch_sensor = MockTouchSensor::new();
        mock_touch_sensor
            .expect_get_touch()
            .times(1)
            .returning(move || Ok(None));
        let mut test_touch_handler = TouchHandler::new(
            mock_touch_sensor,
            Microseconds::<u64>::new(0),
            Some([
                KinState {
                    pos: 0.1,
                    speed: 0.0,
                    accel: 0.0,
                },
                KinState {
                    pos: 0.1,
                    speed: 0.0,
                    accel: 0.0,
                },
            ]),
        );
        let result = test_touch_handler.get_ball_state(Seconds::new(1).into());
        assert!(Ok(None) == result);
    }
    #[test]
    fn test_continued_ball_find() {
        let expected_touch_x = (0.1 / parameter_manager().get::<TouchScaleX>()) as i32
            - parameter_manager().get::<TouchCenterOffsetX>();
        let expected_touch_y = (0.1 / parameter_manager().get::<TouchScaleY>()) as i32
            - parameter_manager().get::<TouchCenterOffsetY>();
        let mut mock_touch_sensor = MockTouchSensor::new();
        mock_touch_sensor
            .expect_get_touch()
            .times(1)
            .returning(move || {
                Ok(Some(Point {
                    x: expected_touch_x,
                    y: expected_touch_y,
                }))
            });
        let mut test_touch_handler = TouchHandler::new(
            mock_touch_sensor,
            Microseconds::<u64>::new(0),
            Some([
                KinState {
                    pos: 0.0,
                    speed: 0.0,
                    accel: 0.0,
                },
                KinState {
                    pos: 0.0,
                    speed: 0.0,
                    accel: 0.0,
                },
            ]),
        );
        let result = test_touch_handler.get_ball_state(Seconds::new(1).into());
        //let expected_result = Ok(Some([KinState{
        //    pos: 0.1,
        //    speed: 0.0,
        //    accel: 0.0,
        //},KinState{
        //    pos: 0.1,
        //    speed: 0.0,
        //    accel: 0.0,
        //}]));
        //assert!(expected_result == result);
    }
}
