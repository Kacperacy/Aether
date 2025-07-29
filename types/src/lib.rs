pub mod bitboard;
mod board_query;
pub mod castling;
pub mod chess_move;
pub mod color;
pub mod file;
mod move_generator;
pub mod piece;
pub mod rank;
pub mod square;
pub mod zobrist_keys;

pub use bitboard::*;
pub use board_query::*;
pub use castling::*;
pub use chess_move::*;
pub use color::*;
pub use file::*;
pub use move_generator::*;
pub use piece::*;
pub use rank::*;
pub use square::*;

pub type Result<T> = std::result::Result<T, &'static str>;
