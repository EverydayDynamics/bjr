
extern crate uom;
use uom::si::f32::*;
use uom::si::length::micrometer;
use uom::si::length::millimeter;
use uom::si::velocity::millimeter_per_second;
pub struct Lga201s06AUecb019 {
    dist_per_step: f32, //mm
    distance_limit: f32, //mm
    velocity_limit: f32, //mm/s
    actuator_length:f32 //mm
}
pub trait LinearStepper {

    fn distance_per_step(&self) -> f32;
    fn distance_limit(&self) -> f32;
    fn velocity_limit(&self) -> f32;
    fn actuator_length(&self) -> f32;
}
impl Lga201s06AUecb019 {
    pub fn new() -> Lga201s06AUecb019 {
        Lga201s06AUecb019 {
            dist_per_step: 0.01,
            distance_limit: 20.0,
            velocity_limit: 60.0,
            actuator_length: 88.0,
        }
    }
}
impl LinearStepper for Lga201s06AUecb019 {
    fn distance_per_step(&self) -> f32 {
       self.dist_per_step
    }
    fn distance_limit(&self) -> f32 {
        self.distance_limit
    }
    fn velocity_limit(&self) -> f32 {
        self.velocity_limit
    }

    fn actuator_length(&self) -> f32 {
        self.actuator_length
    }
}