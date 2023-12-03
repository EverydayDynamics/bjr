
extern crate uom;
use uom::si::f32::*;
use uom::si::length::micrometer;
use uom::si::length::millimeter;
use uom::si::velocity::millimeter_per_second;
pub struct Lga201s06AUecb019 {
    dist_per_step: Length,
    distance_limit: Length,
    velocity_limit: Velocity,
}
pub trait LinearStepper {

    fn distance_per_step(&self) -> Length;
    fn distance_limit(&self) -> Length;
    fn velocity_limit(&self) -> Velocity;
}
impl Lga201s06AUecb019 {
    pub fn new() -> Lga201s06AUecb019 {
        Lga201s06AUecb019 {
            dist_per_step: uom::si::f32::Length::new::<micrometer>(10.0),
            distance_limit: uom::si::f32::Length::new::<millimeter>(20.0),
            velocity_limit: uom::si::f32::Velocity::new::<millimeter_per_second>(60.0),
        }
    }
}
impl LinearStepper for Lga201s06AUecb019 {
    fn distance_per_step(&self) -> Length {
       self.dist_per_step
    }
    fn distance_limit(&self) -> Length {
        self.distance_limit
    }
    fn velocity_limit(&self) -> Velocity {
        self.velocity_limit
    }
}