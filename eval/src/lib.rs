//! Evaluation crate
//!
//! Position evaluation functions for chess positions.

mod simple_evaluator;

use aether_types::Score;
use board::BoardQuery;

/// Trait for evaluating chess positions.
pub trait Evaluator {
    /// Evaluate the given board position and return a score.
    fn evaluate<T: BoardQuery>(&self, board: &T) -> Score;
}
