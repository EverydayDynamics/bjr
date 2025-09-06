#![no_std]
pub trait TimeControl {
    fn get_tick(&self) -> u64;
}
