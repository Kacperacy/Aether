//! Core chess types and traits shared across the workspace.
//!
//! Responsibilities:
//! - Fundamental value types: squares, files, ranks, colors, pieces, bitboards.
//! - Common data structures: moves, castling rights, zobrist keys.
//! - The `BoardQuery` trait to abstract over board state for consumers (e.g., movegen).
//!
//! This crate should remain dependency-light and free of engine/search policy to avoid cycles.

pub mod bitboard;
mod board_query;
pub mod castling;
pub mod chess_move;
pub mod color;
pub mod error;
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
pub use error::*;
pub use file::*;
pub use move_generator::*;
pub use piece::*;
pub use rank::*;
pub use square::*;

/// OR-combines all piece bitboards for a color.
///
/// Convenience helper shared across crates to avoid re-implementing the same micro-utility.
#[inline]
pub fn combine_piece_bitboards(piece_bbs: [BitBoard; 6]) -> BitBoard {
    let [p0, p1, p2, p3, p4, p5] = piece_bbs;
    p0 | p1 | p2 | p3 | p4 | p5
}
