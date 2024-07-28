use core::fmt::{Display, Formatter};
use crate::app::parameter_manager::{MotorLowerPosLimit, MotorLowerVelLimit, MotorUpperPosLimit, MotorUpperVelLimit, parameter_manager, ParameterType};

pub struct LimitChecker {
}
#[derive(PartialEq, Copy, Clone)]
pub struct LimitReport<T>{
    limit: T,
    value: T,
}
impl<T: Display> Display for LimitReport<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f,"LimitReport limit:{}, value: {}", self.limit, self.value)
    }
}
#[derive(PartialEq, Copy, Clone)]
pub enum LimitError<T>{
    OverLimit(LimitReport<T>),
    UnderLimit(LimitReport<T>),
}
impl<T: Display> Display for LimitError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            LimitError::OverLimit(lr) => {write!(f,"LimitError Over limit: {}", lr)}
            LimitError::UnderLimit(lr) => {write!(f,"LimitError Under limit: {}", lr)}
        }
    }
}
impl LimitChecker {
   pub fn new() -> LimitChecker {
       LimitChecker{

       }
   }
}
pub trait Limit<T>
where
    T: PartialOrd + Display
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
    pub fn new(enabled: bool) -> MotorVelLimit {
        MotorVelLimit {enabled}
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_within() {
        let mut test_motor_vel_limit = MotorVelLimit::new(true);
        assert!(test_motor_vel_limit.check(10f32)== Ok(()));
    }
    #[test]
    fn test_limit_above() {
        let mut test_motor_vel_limit = MotorVelLimit::new(true);
        assert!(test_motor_vel_limit.check(1e7) == Err(LimitError::OverLimit(LimitReport{ limit: 1e6, value: 1e7 })));
    }
    #[test]
    fn test_limit_below() {
        let mut test_motor_vel_limit = MotorVelLimit::new(true);
        assert!(test_motor_vel_limit.check(-1e7) == Err(LimitError::UnderLimit(LimitReport{ limit: -1e6, value: -1e7 })));
    }
    #[test]
    fn test_limit_above_disabled() {
        let mut test_motor_vel_limit = MotorVelLimit::new(false);
        assert!(test_motor_vel_limit.check(1e7) == Ok(()));
    }
}
