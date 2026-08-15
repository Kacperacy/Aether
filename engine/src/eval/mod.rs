//! Position evaluation: material, piece-square tables, and the incremental
//! accumulator the search maintains alongside make/unmake.

pub mod accumulator;
pub mod material;
pub mod pst;
pub mod score;
mod simple_evaluator;

pub use accumulator::{EvalState, MAX_GAME_PHASE};
pub use score::{MATE_SCORE, MATE_THRESHOLD, NEG_MATE_SCORE, Score, mated_in, score_to_mate_moves};
pub use simple_evaluator::SimpleEvaluator;

use board::Board;

pub trait Evaluator {
    fn evaluate(&self, board: &Board, eval_state: &EvalState) -> Score;
}
