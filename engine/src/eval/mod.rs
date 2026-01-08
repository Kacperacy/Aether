//! Evaluation crate
//!
//! Position evaluation functions for chess positions

mod simple_evaluator;

pub use simple_evaluator::SimpleEvaluator;

use aether_core::Score;
use board::Board;

/// Trait for evaluating chess positions
pub trait Evaluator {
    /// Evaluate the given board position and return a score
    fn evaluate(&self, board: &Board) -> Score;
}
