//! Core chess vocabulary: the value types every other crate is written in.
//!
//! This crate is deliberately dependency-light and holds no tables, no board
//! state, and no evaluation or search concepts.

pub mod bitboard;
pub mod castling;
pub mod color;
pub mod error;
pub mod file;
pub mod r#move;
pub mod piece;
pub mod rank;
pub mod square;

pub use bitboard::{BitBoard, BitBoardIter};
pub use castling::{CastlingPath, CastlingRights};
pub use color::Color;
pub use error::CoreError;
pub use file::File;
pub use r#move::Move;
pub use piece::Piece;
pub use rank::Rank;
pub use square::Square;

type Result<T> = std::result::Result<T, CoreError>;
