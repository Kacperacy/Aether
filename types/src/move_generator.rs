use crate::{BoardQuery, Move};

pub trait MoveGen {
    fn pseudo_legal<T: BoardQuery>(&self, board: &T, moves: &mut Vec<Move>);
    fn legal<T: BoardQuery>(&self, board: &T, moves: &mut Vec<Move>);
    fn captures<T: BoardQuery>(&self, board: &T, moves: &mut Vec<Move>);
    fn quiet_moves<T: BoardQuery>(&self, board: &T, moves: &mut Vec<Move>);
}