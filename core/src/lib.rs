pub mod attacks;
pub mod bitboard;
pub mod castling;
pub mod chess_move;
pub mod color;
pub mod error;
pub mod file;
pub mod piece;
pub mod rank;
pub mod score;
pub mod square;
pub mod zobrist_keys;

pub use attacks::*;
pub use bitboard::*;
pub use castling::*;
pub use chess_move::*;
pub use color::*;
pub use error::*;
pub use file::*;
pub use piece::*;
pub use rank::*;
pub use score::*;
pub use square::*;
pub use zobrist_keys::*;

type Result<T> = std::result::Result<T, TypeError>;

#[inline]
pub fn combine_piece_bitboards(piece_bbs: [BitBoard; 6]) -> BitBoard {
    let [p0, p1, p2, p3, p4, p5] = piece_bbs;
    p0 | p1 | p2 | p3 | p4 | p5
}
