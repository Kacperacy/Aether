//! Move-legality predicates.
//!
//! These are pure functions of (position, move). The incrementally-maintained
//! inputs they read — checkers, pinners, blockers, king squares — are position
//! state owned by `board`; deciding whether a given move is legal is a
//! move-generation concern and lives here.

use aether_core::{Color, Move, Piece};
use attacks::{is_square_attacked, line_through};
use board::Board;

/// True when playing `mv` would leave (or leave standing) the mover's king in check.
#[inline]
#[must_use]
pub fn would_leave_king_in_check(board: &Board, mv: &Move) -> bool {
    let side = board.side_to_move();

    let moving_piece = board.piece_at(mv.from_sq()).map(|(p, _)| p);
    if moving_piece == Some(Piece::King) {
        return king_move_is_illegal(board, mv, side);
    }

    if mv.is_en_passant() {
        return en_passant_is_illegal(board, mv, side);
    }

    if !board.checkers().is_empty() {
        return king_still_attacked_after(board, mv, side);
    }

    // Not in check and not the king moving: only a pinned piece can expose it,
    // and then only by stepping off the pin ray.
    let from_bb = mv.from_sq().bitboard();
    if (board.blockers_for_king(side) & from_bb).is_empty() {
        return false;
    }

    let king_sq = board.get_king_square(side);
    let pin_line = line_through(king_sq, mv.from_sq());
    (pin_line & mv.to_sq().bitboard()).is_empty()
}

#[inline]
fn king_still_attacked_after(board: &Board, mv: &Move, side: Color) -> bool {
    let opponent = side.opponent();
    let king_sq = board.get_king_square(side);

    let mut occupied = board.occupied();
    occupied &= !mv.from_sq().bitboard();
    occupied |= mv.to_sq().bitboard();

    let mut their_pieces = board.pieces()[opponent as usize];
    if let Some((captured, _)) = board.piece_at(mv.to_sq()) {
        their_pieces[captured as usize] &= !mv.to_sq().bitboard();
    }

    is_square_attacked(king_sq, opponent, occupied, &their_pieces)
}

#[inline]
fn king_move_is_illegal(board: &Board, mv: &Move, side: Color) -> bool {
    let opponent = side.opponent();
    let them = opponent as usize;

    if mv.is_castling() {
        let occupied = (board.occupied() & !mv.from_sq().bitboard()) | mv.to_sq().bitboard();
        return is_square_attacked(mv.to_sq(), opponent, occupied, &board.pieces()[them]);
    }

    let mut occupied = board.occupied();
    occupied &= !mv.from_sq().bitboard();
    occupied |= mv.to_sq().bitboard();

    let mut their_pieces = board.pieces()[them];
    if let Some((captured, _)) = board.piece_at(mv.to_sq()) {
        their_pieces[captured as usize] &= !mv.to_sq().bitboard();
    }

    is_square_attacked(mv.to_sq(), opponent, occupied, &their_pieces)
}

/// En passant is the one move that removes a piece from a square the mover
/// neither leaves nor lands on, so it can uncover a rank attack that no pin
/// mask predicts. It always needs the explicit re-check.
#[inline]
fn en_passant_is_illegal(board: &Board, mv: &Move, side: Color) -> bool {
    let opponent = side.opponent();
    let king_sq = board.get_king_square(side);
    let captured_sq = mv.to_sq().down(side).expect("Invalid en passant");

    let mut occupied = board.occupied();
    occupied &= !mv.from_sq().bitboard();
    occupied &= !captured_sq.bitboard();
    occupied |= mv.to_sq().bitboard();

    let mut their_pieces = board.pieces()[opponent as usize];
    their_pieces[Piece::Pawn as usize] &= !captured_sq.bitboard();

    is_square_attacked(king_sq, opponent, occupied, &their_pieces)
}
