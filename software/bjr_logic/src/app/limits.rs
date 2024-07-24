use crate::app::parameter_manager::{MotorLowerPosLimit, MotorLowerVelLimit, MotorUpperPosLimit, MotorUpperVelLimit, parameter_manager, ParameterType};

pub struct LimitChecker {

}
pub struct LimitReport<T>{
    limit: T,
    value: T,
}
pub enum LimitError<T>{
    OverLimit(LimitReport<T>),
    UnderLimit(LimitReport<T>),
}
impl LimitChecker {
   pub fn new() -> LimitChecker {
       LimitChecker{

       }
   }
}
pub trait Limit<T>
where
    T: PartialOrd
{
    type UpperParam: ParameterType<ReturnType=T>;
    type LowerParam: ParameterType<ReturnType=T>;
    fn enabled(&self) -> bool;
    fn enable(&mut self, enable:bool);
    fn check(&self,  value: T)-> Result<(),LimitError<T>> {
        let upper_limit = parameter_manager().get::<Self::UpperParam>();
        let lower_limit = parameter_manager().get::<Self::LowerParam>();
        if self.enabled() {
            if value > upper_limit {
                return Err(LimitError::OverLimit(LimitReport{limit: upper_limit, value }))
            } else if value < lower_limit {
                return Err(LimitError::UnderLimit(LimitReport{limit: lower_limit, value }))
            }
        }
        Ok(())
    }

}

pub struct MotorPosLimit{
    enabled: bool,
}
impl MotorPosLimit {
    pub fn new(enabled: bool) -> MotorPosLimit {
        MotorPosLimit{enabled}
    }
}
impl Limit<f32> for MotorPosLimit {
    type UpperParam = MotorUpperPosLimit;
    type LowerParam = MotorLowerPosLimit;
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn enable(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
pub struct MotorVelLimit{
    enabled: bool,
}
impl MotorVelLimit {
    pub fn new(enabled: bool) -> MotorPosLimit {
        MotorPosLimit {enabled}
    }
}
impl Limit<f32> for MotorVelLimit {
    type UpperParam = MotorUpperVelLimit;
    type LowerParam = MotorLowerVelLimit;
    fn enabled(&self) -> bool {
        self.enabled
    }
    fn enable(&mut self, enable: bool) {
        self.enabled = enable;
    }
}
