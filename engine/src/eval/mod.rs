mod simple_evaluator;

pub use simple_evaluator::SimpleEvaluator;

use aether_core::Score;
use board::Board;

pub trait Evaluator {
    fn evaluate(&self, board: &Board) -> Score;
}
