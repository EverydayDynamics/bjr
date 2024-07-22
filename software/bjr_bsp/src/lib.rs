#![no_main]
#![no_std]
extern crate alloc;

#[cfg(feature = "mock_board")]
pub mod mock_board;
#[cfg(feature = "main_board")]
pub mod main_board;
mod utils;
mod devices;
pub mod boards;

#[cfg(feature = "mock_board")]
pub use mock_board::MyBoard as Board;
#[cfg(feature = "main_board")]
pub use main_board::MyBoard as Board;
use alloc::alloc::*;

/// The global allocator type.
#[derive(Default)]
pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        todo!()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
    }
}

/// If there is an out of memory error, just panic.
fn my_allocator_error(_layout: Layout) -> ! {
    panic!("out of memory");
}

/// The static global allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;
// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
use defmt_test;
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use bsp_traits::{MotorInput, MotorMode};
    use defmt::assert;
    use crate::boards::BjrBoardSupport;
    use super::*;

    struct State { // state shared between `#[test]` functions
        pub board: Board
    }
    #[init]
    fn init() -> State {
        let board = Board::new().unwrap();
        State{
            board
        }
    }
    #[test]
    fn it_works() {
        assert!(true)
    }
    #[test]
    fn button_use(state: &mut State) {
        let button = state.board.get_button();
        assert_eq!(button.is_pressed(), false);
    }

    #[test]
    fn motor_use(state: &mut State) {
        let mut controllers = state.board.get_stepper_motor_controllers();
        let inputs = MotorInput{
            velocity: 10000,
            acceleration: 10000,
            position: -30000,
            mode: MotorMode::VelocityCtrl,
        };
        let result = controllers[0].set_inputs(inputs).unwrap();
        for i in 1..1000 {
            let result = controllers[0].get_state().unwrap();
            defmt::println!("pos:{}, vel:{}, limit:{}",result.position, result.velocity, result.limit_reached);
        }
        defmt::println!("Swapping over to position mode");
        let inputs = MotorInput{
            velocity: 10000,
            acceleration: 10000,
            position: 0,
            mode: MotorMode::PositionCtrl,
        };
        let result = controllers[0].set_inputs(inputs).unwrap();
        for i in 1..1100 {
            let result = controllers[0].get_state().unwrap();
            defmt::println!("pos:{}, vel:{}, limit:{}",result.position, result.velocity, result.limit_reached);
        }
    }

}
