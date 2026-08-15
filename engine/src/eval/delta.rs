//! Moves expressed as piece additions and removals.
//!
//! Every evaluation that updates incrementally needs the same fact about a move:
//! which pieces left which squares, and which arrived where. Only what it does
//! with that fact differs — a PST evaluator adds table entries, an NNUE
//! evaluator adds and subtracts feature-weight rows.
//!
//! Decoding that once, here, is what lets a network be dropped in later without
//! touching `board` or `search`: the awkward cases (en passant capturing on a
//! square the pawn never occupies, castling moving two pieces, promotion
//! changing piece type) are handled in exactly one place.

use aether_core::{CastlingPath, Color, Move, Piece, Square};
use board::Board;

/// The most piece changes any single move can produce.
///
/// Castling is the worst case at four: the king leaves and arrives, and so does
/// the rook. A capture-promotion is three.
pub const MAX_PIECE_CHANGES: usize = 4;

/// A piece arriving on, or leaving, a square.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PieceChange {
    pub piece: Piece,
    pub square: Square,
    pub color: Color,
    /// True when the piece arrives, false when it leaves.
    pub added: bool,
}

/// The complete effect of one move, as a short sequence of [`PieceChange`]s.
#[derive(Debug, Clone, Copy, Default)]
pub struct MoveDelta {
    changes: [Option<PieceChange>; MAX_PIECE_CHANGES],
    len: usize,
}

impl MoveDelta {
    #[inline]
    fn push(&mut self, piece: Piece, square: Square, color: Color, added: bool) {
        debug_assert!(self.len < MAX_PIECE_CHANGES, "move delta overflow");
        if self.len < MAX_PIECE_CHANGES {
            self.changes[self.len] = Some(PieceChange {
                piece,
                square,
                color,
                added,
            });
            self.len += 1;
        }
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = PieceChange> + '_ {
        self.changes[..self.len].iter().filter_map(|c| *c)
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Decode `mv` into the pieces it removes and adds.
///
/// Must be called **before** `board.make_move(mv)`: it reads the pre-move
/// position to find the moving piece and what is captured.
///
/// Returns `None` when `mv` has no piece on its origin square, which the search
/// treats as a move to skip rather than an error.
///
/// Removals are emitted before additions. That ordering matters for a consumer
/// that clamps as it goes, and it keeps a capture-promotion (remove pawn, remove
/// victim, add the new piece) from ever double-counting the destination square.
#[must_use]
pub fn decode_move(board: &Board, mv: &Move) -> Option<MoveDelta> {
    let side = board.side_to_move();
    let opponent = side.opponent();
    let (moving_piece, _) = board.piece_at(mv.from_sq())?;

    let mut delta = MoveDelta::default();

    // The mover leaves its origin.
    delta.push(moving_piece, mv.from_sq(), side, false);

    // The victim leaves — for en passant, from a square the capturer never
    // occupies, which is the whole reason this is centralised.
    if mv.is_en_passant() {
        let captured_sq = mv.to_sq().down(side).expect("invalid en passant square");
        delta.push(Piece::Pawn, captured_sq, opponent, false);
    } else if mv.is_capture()
        && let Some((captured, _)) = board.piece_at(mv.to_sq())
    {
        delta.push(captured, mv.to_sq(), opponent, false);
    }

    // The mover arrives, as whatever it promoted to.
    let arriving = mv.promotion_piece().unwrap_or(moving_piece);
    delta.push(arriving, mv.to_sq(), side, true);

    // Castling moves the rook too.
    if mv.is_castling()
        && let Some(path) = CastlingPath::for_king_destination(side, mv.to_sq())
    {
        delta.push(Piece::Rook, path.rook_from, side, false);
        delta.push(Piece::Rook, path.rook_to, side, true);
    }

    Some(delta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::test_support::pos;

    fn decode(fen: &str, mv: Move) -> MoveDelta {
        decode_move(&pos(fen), &mv).expect("piece on origin square")
    }

    fn count(delta: &MoveDelta) -> (usize, usize) {
        (
            delta.iter().filter(|c| !c.added).count(),
            delta.iter().filter(|c| c.added).count(),
        )
    }

    #[test]
    fn test_quiet_move_is_one_removal_and_one_addition() {
        let d = decode(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH),
        );
        assert_eq!(count(&d), (1, 1));
    }

    #[test]
    fn test_capture_removes_the_victim() {
        let d = decode(
            "7k/8/8/3q4/4P3/8/8/K7 w - - 0 1",
            Move::new(Square::E4, Square::D5, Move::CAPTURE),
        );
        assert_eq!(count(&d), (2, 1));
        assert!(
            d.iter()
                .any(|c| !c.added && c.piece == Piece::Queen && c.square == Square::D5)
        );
    }

    /// En passant is the case a naive decode gets wrong: the captured pawn is
    /// not on the destination square.
    #[test]
    fn test_en_passant_removes_from_the_pawn_square_not_the_destination() {
        let d = decode(
            "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
            Move::new(Square::E5, Square::F6, Move::EN_PASSANT),
        );
        assert_eq!(count(&d), (2, 1));
        assert!(
            d.iter()
                .any(|c| !c.added && c.piece == Piece::Pawn && c.square == Square::F5),
            "captured pawn should be removed from f5, not f6"
        );
    }

    #[test]
    fn test_promotion_adds_the_promoted_piece_not_a_pawn() {
        let d = decode(
            "7k/4P3/8/8/8/8/8/K7 w - - 0 1",
            Move::new(Square::E7, Square::E8, Move::PROMO_Q),
        );
        assert_eq!(count(&d), (1, 1));
        assert!(
            d.iter()
                .any(|c| c.added && c.piece == Piece::Queen && c.square == Square::E8)
        );
        assert!(d.iter().any(|c| !c.added && c.piece == Piece::Pawn));
    }

    #[test]
    fn test_castling_moves_both_king_and_rook() {
        let d = decode(
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            Move::new(Square::E1, Square::G1, Move::CASTLE_KS),
        );
        assert_eq!(count(&d), (2, 2));
        assert_eq!(d.len(), MAX_PIECE_CHANGES);
        assert!(
            d.iter()
                .any(|c| c.added && c.piece == Piece::Rook && c.square == Square::F1)
        );
        assert!(
            d.iter()
                .any(|c| !c.added && c.piece == Piece::Rook && c.square == Square::H1)
        );
    }

    #[test]
    fn test_empty_origin_square_decodes_to_nothing() {
        let board = pos("7k/8/8/8/8/8/8/K7 w - - 0 1");
        assert!(decode_move(&board, &Move::new(Square::E4, Square::E5, Move::QUIET)).is_none());
    }
}
