pub mod attacks;
pub mod bitboard;
pub mod castling;
pub mod color;
pub mod error;
pub mod file;
pub mod r#move;
pub mod piece;
pub mod rank;
pub mod score;
pub mod square;
pub mod zobrist_keys;

pub use attacks::*;
pub use bitboard::*;
pub use castling::*;
pub use color::*;
pub use error::*;
pub use file::*;
pub use r#move::*;
pub use piece::*;
pub use rank::*;
pub use score::*;
pub use square::*;
pub use zobrist_keys::*;

type Result<T> = std::result::Result<T, TypeError>;
