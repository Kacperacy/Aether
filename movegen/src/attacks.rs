use crate::magic::{get_bishop_attacks, get_rook_attacks};
use crate::pieces::get_king_moves;
use crate::pieces::knight::get_knight_attacks;
use crate::pieces::pawn::get_pawn_attacks_to_square;
use aether_types::{BitBoard, Color, Piece, Square};

/// Shared low-level attack/occupancy helpers used across crates.
/// Centralizing these avoids duplication and keeps attack logic consistent.

/// Returns a bitboard of pieces (of `color`) that attack `sq`, using the provided
/// board occupancy `occ` and that color's piece bitboards `their`.
#[inline]
pub fn attackers_to_square_with_occ(
    sq: Square,
    color: Color,
    occ: BitBoard,
    their: &[BitBoard; 6],
) -> BitBoard {
    (get_pawn_attacks_to_square(sq, color) & their[Piece::Pawn as usize])
        | (get_knight_attacks(sq) & their[Piece::Knight as usize])
        | (get_bishop_attacks(sq, occ)
            & (their[Piece::Bishop as usize] | their[Piece::Queen as usize]))
        | (get_rook_attacks(sq, occ) & (their[Piece::Rook as usize] | their[Piece::Queen as usize]))
        | (get_king_moves(sq) & their[Piece::King as usize])
}
