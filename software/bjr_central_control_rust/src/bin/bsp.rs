
#[cfg_attr(not(feature = "mock_board"), no_main)]
#[cfg_attr(not(feature = "mock_board"), no_std)]

use bjr as _;
use bjr::bsp;
use bjr::bsp::traits::BoardSupport; // global logger + panicking-behavior + memory layout
use bsp::Board;
use bjr::app::button_handler::ButtonHandler;
use heapless::mpmc::Q8;
use bjr::app::event::GlobEvent;
static EVENTQUEUE : Q8<GlobEvent> = Q8::new();

#[cortex_m_rt::entry]
fn main() -> ! {

    {
        let mut board = Board::new();
        let button_handler = ButtonHandler::new(board.get_button(), &EVENTQUEUE);
        let temp_sensor = board.get_temperature_sensor();
    }
    defmt::println!("BSP loaded!");

    bjr::exit()
}
#[cfg(feature = "mock_board")]
fn main() -> ! {loop{}}
