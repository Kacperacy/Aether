//! Move-legality predicates.
//!
//! These are pure functions of (position, move). The incrementally-maintained
//! inputs they read — checkers, pinners, blockers, king squares — are position
//! state owned by `board`; deciding whether a given move is legal is a
//! move-generation concern and lives here.

use aether_core::{CastlingPath, Color, Move, Piece};
use attacks::{
    bishop_attacks, is_square_attacked, king_attacks, knight_attacks, line_through, pawn_attacks,
    pawn_moves, queen_attacks, rook_attacks,
};
use board::Board;

/// True when `mv` is a legal move in `board`, for an *arbitrary* move encoding.
///
/// Everything else in this module assumes its input came out of the generator.
/// This does not: it validates the piece, the geometry, every flag bit and the
/// king's safety from scratch, so it is safe to call on a move that arrived from
/// outside — a transposition-table hit after a key collision, or a UCI string.
///
/// The move picker depends on this. It yields the TT move *before* generating
/// anything, so nothing else can vouch for that move; previously a bogus TT move
/// was harmless only because it had to match a generated move to be played.
///
/// Flags are checked exactly, not leniently. Generation emits exactly one
/// encoding per move and the search compares moves by their bits, so a move that
/// is right about from/to but wrong about, say, its capture bit is *not* the same
/// move — accepting it would let the picker yield a duplicate that no later stage
/// can recognise and skip.
#[must_use]
pub fn is_legal(board: &Board, mv: &Move) -> bool {
    let from = mv.from_sq();
    let to = mv.to_sq();

    // Rejects `Move::NULL`, which encodes a1a1.
    if from == to {
        return false;
    }

    let side = board.side_to_move();
    let Some((piece, owner)) = board.piece_at(from) else {
        return false;
    };
    if owner != side || board.occupied_by(side).contains(to) {
        return false;
    }

    if !shape_is_pseudo_legal(board, mv, piece, side) {
        return false;
    }

    !would_leave_king_in_check(board, mv)
}

/// Does `mv` describe a move this piece can physically make in this position,
/// ignoring king safety?
fn shape_is_pseudo_legal(board: &Board, mv: &Move, piece: Piece, side: Color) -> bool {
    let from = mv.from_sq();
    let to = mv.to_sq();
    let flags = mv.flags();

    match flags {
        Move::EN_PASSANT => {
            piece == Piece::Pawn
                && board.en_passant_square() == Some(to)
                && pawn_attacks(from, side).contains(to)
        }

        Move::CASTLE_KS | Move::CASTLE_QS => {
            piece == Piece::King && castling_is_available(board, mv, side)
        }

        _ if piece == Piece::Pawn => pawn_shape_is_pseudo_legal(board, mv, side),

        // Every other piece has exactly two legal encodings. Anything else —
        // a promotion bit, a double-push bit, or one of the two unused flag
        // values (6 and 7) — is a malformed move, not a lenient one.
        Move::QUIET | Move::CAPTURE => {
            let occupied = board.occupied();
            if mv.is_capture() != board.occupied_by(side.opponent()).contains(to) {
                return false;
            }
            let attacks = match piece {
                Piece::Knight => knight_attacks(from),
                Piece::Bishop => bishop_attacks(from, occupied),
                Piece::Rook => rook_attacks(from, occupied),
                Piece::Queen => queen_attacks(from, occupied),
                Piece::King => king_attacks(from),
                Piece::Pawn => return false,
            };
            attacks.contains(to)
        }

        _ => false,
    }
}

fn pawn_shape_is_pseudo_legal(board: &Board, mv: &Move, side: Color) -> bool {
    let from = mv.from_sq();
    let to = mv.to_sq();
    let occupied = board.occupied();

    let is_capture =
        pawn_attacks(from, side).contains(to) && board.occupied_by(side.opponent()).contains(to);
    let is_push = !occupied.contains(to) && pawn_moves(from, side, occupied).contains(to);

    if to.is_promotion_rank(side) {
        return match mv.flags() {
            Move::PROMO_N..=Move::PROMO_Q => is_push,
            Move::PROMO_CAP_N..=Move::PROMO_CAP_Q => is_capture,
            _ => false,
        };
    }

    let is_double = to.rank().to_index().abs_diff(from.rank().to_index()) == 2;
    match mv.flags() {
        Move::QUIET => is_push && !is_double,
        Move::DOUBLE_PUSH => is_push && is_double,
        Move::CAPTURE => is_capture,
        _ => false,
    }
}

/// Mirrors the generator's castling test: the rights, the king's own square, a
/// clear path, and no attacked square along the king's route. The destination
/// square itself is covered by [`would_leave_king_in_check`].
fn castling_is_available(board: &Board, mv: &Move, side: Color) -> bool {
    let (has_rights, path) = if mv.flags() == Move::CASTLE_KS {
        (board.can_castle_short(side), CastlingPath::kingside(side))
    } else {
        (board.can_castle_long(side), CastlingPath::queenside(side))
    };

    has_rights
        && mv.from_sq() == path.king_from
        && mv.to_sq() == path.king_to
        && path.vacancy.iter().all(|&sq| !board.is_square_occupied(sq))
        && path
            .king_safety
            .iter()
            .all(|&sq| !board.is_square_attacked(sq, side.opponent()))
}

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
