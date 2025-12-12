//! Polyglot-compatible Zobrist hash calculation
//!
//! This module provides hash calculation compatible with the Polyglot
//! opening book format (.bin files).

mod polyglot_keys;

use crate::{Board, BoardQuery};
use aether_core::{Color, File, Piece, Rank, Square};
use polyglot_keys::{
    POLYGLOT_RANDOM_ARRAY, RANDOM_CASTLE_OFFSET, RANDOM_EP_OFFSET, RANDOM_TURN_OFFSET,
    polyglot_piece_index, polyglot_piece_kind,
};

/// Calculate the Polyglot-compatible Zobrist hash for a board position.
///
/// This hash is compatible with the Polyglot opening book format and uses
/// the standard Polyglot random array for reproducible results across
/// different chess programs.
///
/// Key differences from a typical Zobrist implementation:
/// - XOR turn key when WHITE to move (not black)
/// - En passant is only hashed if a pawn can actually capture
/// - Uses specific piece encoding: bp=0, wp=1, bn=2, wn=3, bb=4, wb=5, br=6, wr=7, bq=8, wq=9, bk=10, wk=11
pub fn polyglot_hash(board: &Board) -> u64 {
    let mut hash = 0u64;

    // Hash all pieces on the board
    for rank in 0i8..8 {
        for file in 0i8..8 {
            let square = Square::new(File::from_index(file), Rank::from_index(rank));

            if let Some((piece, color)) = board.piece_at(square) {
                let piece_type = piece as usize;
                let is_white = color == Color::White;
                let piece_kind = polyglot_piece_kind(piece_type, is_white);
                let index = polyglot_piece_index(piece_kind, rank as usize, file as usize);
                hash ^= POLYGLOT_RANDOM_ARRAY[index];
            }
        }
    }

    // Hash castling rights (indices 768-771)
    // Order: white kingside, white queenside, black kingside, black queenside
    if board.can_castle_short(Color::White) {
        hash ^= POLYGLOT_RANDOM_ARRAY[RANDOM_CASTLE_OFFSET];
    }
    if board.can_castle_long(Color::White) {
        hash ^= POLYGLOT_RANDOM_ARRAY[RANDOM_CASTLE_OFFSET + 1];
    }
    if board.can_castle_short(Color::Black) {
        hash ^= POLYGLOT_RANDOM_ARRAY[RANDOM_CASTLE_OFFSET + 2];
    }
    if board.can_castle_long(Color::Black) {
        hash ^= POLYGLOT_RANDOM_ARRAY[RANDOM_CASTLE_OFFSET + 3];
    }

    // Hash en passant - ONLY if a pawn can actually capture!
    if let Some(ep_square) = board.en_passant_square() {
        if can_pawn_capture_en_passant(board, ep_square) {
            let file = ep_square.file() as usize;
            hash ^= POLYGLOT_RANDOM_ARRAY[RANDOM_EP_OFFSET + file];
        }
    }

    // Hash side to move - XOR when WHITE to move (Polyglot convention)
    if board.side_to_move() == Color::White {
        hash ^= POLYGLOT_RANDOM_ARRAY[RANDOM_TURN_OFFSET];
    }

    hash
}

/// Check if there's an enemy pawn that can capture en passant
fn can_pawn_capture_en_passant(board: &Board, ep_square: Square) -> bool {
    let side_to_move = board.side_to_move();
    let ep_file = ep_square.file() as i8;

    // The capturing pawn must be on rank 5 (for white, 0-indexed: 4) or rank 4 (for black, 0-indexed: 3)
    // and adjacent to the ep file
    let pawn_rank: i8 = if side_to_move == Color::White { 4 } else { 3 };

    // Check left adjacent square
    if ep_file > 0 {
        let left_file = ep_file - 1;
        let check_square = Square::new(File::from_index(left_file), Rank::from_index(pawn_rank));
        if let Some((piece, color)) = board.piece_at(check_square) {
            if piece == Piece::Pawn && color == side_to_move {
                return true;
            }
        }
    }

    // Check right adjacent square
    if ep_file < 7 {
        let right_file = ep_file + 1;
        let check_square = Square::new(File::from_index(right_file), Rank::from_index(pawn_rank));
        if let Some((piece, color)) = board.piece_at(check_square) {
            if piece == Piece::Pawn && color == side_to_move {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FenOps;

    #[test]
    fn test_polyglot_hash_starting_position() {
        // Known Polyglot hash for the starting position
        const STARTING_POSITION_KEY: u64 = 0x463b96181691fc9c;

        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let hash = polyglot_hash(&board);

        assert_eq!(
            hash, STARTING_POSITION_KEY,
            "Starting position hash mismatch: got 0x{:016x}, expected 0x{:016x}",
            hash, STARTING_POSITION_KEY
        );
    }

    #[test]
    fn test_polyglot_hash_after_e4() {
        // Known Polyglot hash for position after 1.e4
        const AFTER_E4_KEY: u64 = 0x823c9b50fd114196;

        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();
        let hash = polyglot_hash(&board);

        assert_eq!(
            hash, AFTER_E4_KEY,
            "After 1.e4 hash mismatch: got 0x{:016x}, expected 0x{:016x}",
            hash, AFTER_E4_KEY
        );
    }
}
