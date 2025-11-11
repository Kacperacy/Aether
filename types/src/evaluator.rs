use crate::BoardQuery;

pub type Score = i32;

pub const MATE_SCORE: Score = 100_000;
pub const NEG_MATE_SCORE: Score = -100_000;

/// Returns a score indicating mate in `n` moves.
pub const fn mate_in(n: u32) -> Score {
    MATE_SCORE - (n as Score)
}

/// Returns a score indicating being mated in `n` moves.
pub const fn mated_in(n: u32) -> Score {
    NEG_MATE_SCORE + (n as Score)
}

/// Trait for evaluating chess positions.
pub trait Evaluator {
    fn evaluate<T: BoardQuery>(&self, board: &T) -> Score;
}
