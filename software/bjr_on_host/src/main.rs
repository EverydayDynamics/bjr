use bjr_bsp::Board;
use bjr_builder::build_application;
use bjr_logic::app::menu_handler::MenuContext;
use embedded_time::duration::*;
use os_traits::TimeControl;
use std::thread;
use std::time::{Duration, Instant};
struct StdTimeControl {
    pub start_time: Instant,
}
impl StdTimeControl {
    pub fn new() -> StdTimeControl {
        StdTimeControl {
            start_time: Instant::now(),
        }
    }
}
impl TimeControl for StdTimeControl {
    fn get_tick(&self) -> u64 {
        (Instant::now() - self.start_time).as_micros() as u64
    }
}

#[feature(thread_sleep_until)]
fn main() {
    let std_time_control = StdTimeControl::new();
    let start_time = std_time_control.start_time;
    let mut board = Board::new();
    let time = (Instant::now() - start_time).as_micros() as u64;
    let mut menu_context = MenuContext::default();
    let mut logic_runner = build_application(&mut board, &mut menu_context, std_time_control);
    loop {
        let time = (Instant::now() - start_time).as_micros() as u64;
        let next_run = logic_runner.update();
        let time = (Instant::now() - start_time).as_micros() as u64;
        thread::sleep(Duration::from_micros(
            next_run.integer().saturating_sub(time),
        ));
    }
}
