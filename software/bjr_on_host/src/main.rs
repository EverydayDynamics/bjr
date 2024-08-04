use bjr_builder::build_application;
use bjr_bsp::Board;
use embedded_time::duration::*;
use std::time::{Duration, Instant};
use std::thread;
#[feature(thread_sleep_until)]
fn main() {

    //let start_time = Instant::now();
    //let mut board = Board::new();
    //let time = (Instant::now() - start_time).as_micros() as u64;
    //let mut logic_runner = build_application(time, &mut board);
    //loop {
    //    let time = (Instant::now() - start_time).as_micros() as u64;
    //    let next_run = logic_runner.update(Microseconds::new(time));
    //    let time = (Instant::now() - start_time).as_micros() as u64;
    //    thread::sleep(Duration::from_micros(next_run.integer()-time));
    //}

}
