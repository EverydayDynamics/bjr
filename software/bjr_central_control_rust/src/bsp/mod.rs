pub mod traits;
mod devices;
#[cfg(feature = "mock_board")]
pub mod mock_board;
#[cfg(feature = "real_board")]
pub mod real_board;

#[cfg(feature = "mock_board")]
pub use mock_board::MyBoard as Board;
#[cfg(feature = "real_board")]
pub use real_board::MyBoard as Board;
