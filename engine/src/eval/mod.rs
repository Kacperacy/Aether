//! Position evaluation: material, piece-square tables, and the incremental
//! accumulator the search maintains alongside make/unmake.

pub mod accumulator;
pub mod delta;
pub mod material;
pub mod pst;
pub mod score;
mod simple_evaluator;

pub use accumulator::{EvalState, MAX_GAME_PHASE};
pub use delta::{MAX_PIECE_CHANGES, MoveDelta, PieceChange, decode_move};
pub use score::{MATE_SCORE, MATE_THRESHOLD, NEG_MATE_SCORE, Score, mated_in, score_to_mate_moves};
pub use simple_evaluator::SimpleEvaluator;

use board::Board;

use aether_core::Move;

/// Incrementally-maintained state an [`Evaluator`] keeps in step with make and
/// unmake.
///
/// The search drives this without knowing what is inside: it pushes before every
/// `make_move` and pops after every `unmake_move`. That discipline is the part
/// that is easy to get wrong and is already correct, so an NNUE accumulator
/// inherits it by implementing this trait rather than by reimplementing it.
pub trait Accumulator {
    /// A zeroed accumulator, for building a searcher before a position exists.
    /// [`Accumulator::reset`] must run before any evaluation.
    fn empty() -> Self;

    /// Re-seed from `board`, discarding any pending frames.
    fn reset(&mut self, board: &Board);

    /// Fold in `mv`'s delta. Called **before** `board.make_move(mv)`, since it
    /// reads the pre-move position.
    fn push(&mut self, board: &Board, mv: &Move);

    /// Push a frame for a null move, which changes no piece placement.
    fn push_null(&mut self);

    /// Undo the most recent push.
    fn pop(&mut self);
}

pub trait Evaluator {
    /// The state this evaluator needs maintained across make/unmake. A
    /// hand-crafted evaluator wants running PST sums; a network wants its
    /// hidden-layer accumulator. The search stores whichever this names.
    type Acc: Accumulator;

    fn evaluate(&self, board: &Board, acc: &Self::Acc) -> Score;
}
