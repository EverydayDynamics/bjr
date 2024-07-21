#![no_main]
#![no_std]




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

// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use bsp_traits::BoardSupport;
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
        let controllers = state.board.get_stepper_motor_controllers();
    }

}
