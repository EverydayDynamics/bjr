use bsp_traits::{TouchSensor, TouchSensorError};
use crate::app::control_primitives::KinState;
use crate::app::parameter_manager::{parameter_manager, TouchCenterOffsetX, TouchCenterOffsetY, TouchScaleX, TouchScaleY};

pub struct TouchHandler<TS> {
    touch_sensor: TS,
    last_state: Option<[KinState;2]>,
    last_call_time: u64,
}
impl<TS> TouchHandler<TS>
where TS: TouchSensor
{
    pub fn new(touch_sensor: TS) -> TouchHandler<TS> {
        TouchHandler{
            touch_sensor,
            last_state: None,
            last_call_time: 0,
        }

    }
    fn calc_kin(&mut self, pos: f32, call_time: u64, last_state: KinState) -> KinState {
        let delta_t = (call_time - self.last_call_time)as f32;
        let speed = (pos - last_state.pos)/delta_t;
        let accel = (speed - last_state.speed)/delta_t;
        let state: KinState = KinState{
            pos,
            speed,
            accel,
        };

       state
    }
    pub fn get_ball_state(&mut self, call_time: u64) -> Result<Option<[KinState;2]>,TouchSensorError> {
        let mut result = Ok(None);
        if let Some(touch_pos) = self.touch_sensor.get_touch()? {
            let x_offset = parameter_manager().get::<TouchCenterOffsetX>();
            let y_offset = parameter_manager().get::<TouchCenterOffsetY>();
            let x_scale = parameter_manager().get::<TouchScaleX>();
            let y_scale = parameter_manager().get::<TouchScaleY>();
            let x_pos = (touch_pos.x + x_offset) as f32 * x_scale;
            let y_pos = (touch_pos.y + y_offset) as f32 * y_scale;
            let mut new_sate = None;
            if let Some(last_state) = self.last_state {
                let xkinstate = self.calc_kin(x_pos, call_time, last_state[0]);
                let ykinstate = self.calc_kin(y_pos, call_time, last_state[1]);
                new_sate = Some([xkinstate, ykinstate]);
            } else {
                let x_trivial_state = KinState {
                    pos: x_pos,
                    speed: 0.0,
                    accel: 0.0,
                };
                let y_trivial_state = KinState {
                    pos: x_pos,
                    speed: 0.0,
                    accel: 0.0,
                };
                new_sate = Some([x_trivial_state, y_trivial_state]);
            }
            self.last_state = new_sate;
            self.last_call_time = call_time;

        }
        result
    }
}