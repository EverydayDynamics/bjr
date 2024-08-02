use bjr_builder::build_application;
use bjr_bsp::Board;
use embedded_time::duration::*;
fn main() {
    let mut board = Board::new();
    let mut logic_runner = build_application(0, &mut board);
    loop {
        let next_run = logic_runner.update(Microseconds::new(0));

    }

}
