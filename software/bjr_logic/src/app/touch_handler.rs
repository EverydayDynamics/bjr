use bsp_traits::TouchSensor;

pub struct TouchHandler<TS> {
    touch_sensor: TS,
}
impl<TS> TouchHandler<TS>
where TS: TouchSensor
{
    pub fn new(touch_sensor: TS) -> TouchHandler<TS> {
        TouchHandler{touch_sensor}
    }
}