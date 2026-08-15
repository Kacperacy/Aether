//! Incrementally-maintained evaluation state.
//!
//! The board crate stores chess state only; the running PST/material sums and
//! the game-phase counter are evaluation concepts and live here. The search
//! pushes a delta before each `make_move` and pops it after each `unmake_move`,
//! so paths that never evaluate (perft, plain move generation) pay nothing.

use super::pst;
use aether_core::{CastlingPath, Move, Piece};
use board::Board;

pub const MAX_GAME_PHASE: i32 = 256;

const PHASE_KNIGHT: i16 = 1;
const PHASE_BISHOP: i16 = 1;
const PHASE_ROOK: i16 = 2;
const PHASE_QUEEN: i16 = 4;
const PHASE_TOTAL: i16 = 24; // 4*1 + 4*1 + 4*2 + 2*4

#[inline]
const fn phase_weight(piece: Piece) -> i16 {
    match piece {
        Piece::Knight => PHASE_KNIGHT,
        Piece::Bishop => PHASE_BISHOP,
        Piece::Rook => PHASE_ROOK,
        Piece::Queen => PHASE_QUEEN,
        Piece::Pawn | Piece::King => 0,
    }
}

#[inline]
const fn phase_delta(piece: Piece) -> i16 {
    (phase_weight(piece) as i32 * MAX_GAME_PHASE / PHASE_TOTAL as i32) as i16
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Frame {
    mg: i32,
    eg: i32,
    phase: i16,
}

/// Running middlegame/endgame PST+material sums and the game phase.
#[derive(Debug, Clone)]
pub struct EvalState {
    current: Frame,
    stack: Vec<Frame>,
}

impl EvalState {
    /// A zeroed accumulator, for constructing a searcher before a position is
    /// known. [`EvalState::reset`] must run before any evaluation.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            current: Frame::default(),
            stack: Vec::with_capacity(crate::search::MAX_PLY),
        }
    }

    /// Re-seed from `board`, discarding any pending frames.
    pub fn reset(&mut self, board: &Board) {
        let (mg, eg) = pst::compute_pst_score(board.pieces());
        self.current = Frame {
            mg,
            eg,
            phase: compute_game_phase(board),
        };
        self.stack.clear();
    }

    #[inline(always)]
    #[must_use]
    pub const fn scores(&self) -> (i32, i32) {
        (self.current.mg, self.current.eg)
    }

    #[inline(always)]
    #[must_use]
    pub const fn game_phase(&self) -> i32 {
        self.current.phase as i32
    }

    /// Apply `mv`'s evaluation delta. Must be called **before**
    /// `board.make_move(mv)` — it reads the pre-move position.
    pub fn push(&mut self, board: &Board, mv: &Move) {
        self.stack.push(self.current);

        let side = board.side_to_move();
        let opponent = side.opponent();

        let Some((moving_piece, _)) = board.piece_at(mv.from_sq()) else {
            return;
        };

        let captured_piece = if mv.is_en_passant() {
            Some(Piece::Pawn)
        } else if mv.is_capture() {
            board.piece_at(mv.to_sq()).map(|(p, _)| p)
        } else {
            None
        };

        if let Some(captured) = captured_piece {
            self.current.phase = (self.current.phase - phase_delta(captured)).max(0);
        }

        if let Some(promo) = mv.promotion_piece() {
            self.current.phase =
                (self.current.phase + phase_delta(promo)).min(MAX_GAME_PHASE as i16);
        }

        let (from_mg, from_eg) = pst::piece_value(moving_piece, mv.from_sq(), side);
        self.current.mg -= from_mg;
        self.current.eg -= from_eg;

        if let Some(captured) = captured_piece {
            let (cap_mg, cap_eg) = if mv.is_en_passant() {
                let captured_sq = mv.to_sq().down(side).expect("Invalid en passant square");
                pst::piece_value(Piece::Pawn, captured_sq, opponent)
            } else {
                pst::piece_value(captured, mv.to_sq(), opponent)
            };
            self.current.mg -= cap_mg;
            self.current.eg -= cap_eg;
        }

        let final_piece = mv.promotion_piece().unwrap_or(moving_piece);
        let (to_mg, to_eg) = pst::piece_value(final_piece, mv.to_sq(), side);
        self.current.mg += to_mg;
        self.current.eg += to_eg;

        if mv.is_castling()
            && let Some(path) = CastlingPath::for_king_destination(side, mv.to_sq())
        {
            let (rf_mg, rf_eg) = pst::piece_value(Piece::Rook, path.rook_from, side);
            let (rt_mg, rt_eg) = pst::piece_value(Piece::Rook, path.rook_to, side);
            self.current.mg += rt_mg - rf_mg;
            self.current.eg += rt_eg - rf_eg;
        }
    }

    /// A null move leaves material and PST untouched; we still push a frame so
    /// that [`EvalState::pop`] stays symmetric with the search's unmake.
    #[inline]
    pub fn push_null(&mut self) {
        self.stack.push(self.current);
    }

    /// Undo the most recent [`EvalState::push`] / [`EvalState::push_null`].
    #[inline]
    pub fn pop(&mut self) {
        if let Some(frame) = self.stack.pop() {
            self.current = frame;
        }
    }
}

fn compute_game_phase(board: &Board) -> i16 {
    use aether_core::Color;

    let mut material = 0i32;
    for color in Color::ALL {
        for piece in [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen] {
            let count = board.piece_count(piece, color) as i32;
            material += count * phase_weight(piece) as i32;
        }
    }

    ((material * MAX_GAME_PHASE) / PHASE_TOTAL as i32).min(MAX_GAME_PHASE) as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use board::STARTING_POSITION_FEN;

    /// Full from-scratch recompute, to check the incremental path against.
    fn recompute(board: &Board) -> EvalState {
        let mut state = EvalState::empty();
        state.reset(board);
        state
    }

    /// The incremental path must agree with a from-scratch recompute after any
    /// sequence of make/unmake — this is the invariant the board crate used to
    /// guarantee internally.
    fn assert_incremental_matches_full(fen: &str, moves: &[Move]) {
        let mut board: Board = fen.parse().unwrap();
        let mut acc = recompute(&board);

        for mv in moves {
            acc.push(&board, mv);
            board.make_move(mv).unwrap();

            let fresh = recompute(&board);
            assert_eq!(acc.scores(), fresh.scores(), "pst mismatch after {mv}");
            assert_eq!(
                acc.game_phase(),
                fresh.game_phase(),
                "phase mismatch after {mv}"
            );
        }

        for mv in moves.iter().rev() {
            board.unmake_move(mv).unwrap();
            acc.pop();

            let fresh = recompute(&board);
            assert_eq!(acc.scores(), fresh.scores(), "pst mismatch unmaking {mv}");
            assert_eq!(
                acc.game_phase(),
                fresh.game_phase(),
                "phase mismatch unmaking {mv}"
            );
        }
    }

    #[test]
    fn test_quiet_and_double_push() {
        use aether_core::Square;
        assert_incremental_matches_full(
            STARTING_POSITION_FEN,
            &[
                Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH),
                Move::new(Square::E7, Square::E5, Move::DOUBLE_PUSH),
                Move::new(Square::G1, Square::F3, Move::QUIET),
            ],
        );
    }

    #[test]
    fn test_capture() {
        use aether_core::Square;
        assert_incremental_matches_full(
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2",
            &[Move::new(Square::E4, Square::D5, Move::CAPTURE)],
        );
    }

    #[test]
    fn test_en_passant() {
        use aether_core::Square;
        assert_incremental_matches_full(
            "rnbqkbnr/pppp1ppp/8/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 1",
            &[Move::new(Square::D5, Square::E6, Move::EN_PASSANT)],
        );
    }

    #[test]
    fn test_promotion_and_promo_capture() {
        use aether_core::Square;
        assert_incremental_matches_full(
            "8/P7/8/8/8/8/8/4K2k w - - 0 1",
            &[Move::new(Square::A7, Square::A8, Move::PROMO_Q)],
        );
        assert_incremental_matches_full(
            "r3k2r/pPpppppp/8/8/8/8/P1PPPPPP/R3K2R w KQkq - 0 1",
            &[Move::new(Square::B7, Square::A8, Move::PROMO_CAP_Q)],
        );
    }

    #[test]
    fn test_castling_both_sides() {
        use aether_core::Square;
        assert_incremental_matches_full(
            "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1",
            &[
                Move::new(Square::E1, Square::G1, Move::CASTLE_KS),
                Move::new(Square::E8, Square::C8, Move::CASTLE_QS),
            ],
        );
    }

    #[test]
    fn test_null_move_is_neutral() {
        let board: Board = STARTING_POSITION_FEN.parse().unwrap();
        let mut acc = recompute(&board);
        let before = (acc.scores(), acc.game_phase());

        acc.push_null();
        assert_eq!((acc.scores(), acc.game_phase()), before);
        acc.pop();
        assert_eq!((acc.scores(), acc.game_phase()), before);
    }
}
