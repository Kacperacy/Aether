use crate::{BoardQuery, Move};

pub trait MoveGenerator {
    fn generate_moves<T: BoardQuery>(&self, board: &T, moves: &mut Vec<Move>);
    fn is_legal_move<T: BoardQuery>(&self, board: &T, mv: Move) -> bool;
}
