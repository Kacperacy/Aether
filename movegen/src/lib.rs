pub mod generator;
use aether_core::Move;
use board::BoardQuery;
pub use generator::Generator;

pub trait MoveGen<T: BoardQuery> {
    /// Fills `moves` with all pseudo-legal moves for `board`.
    fn pseudo_legal(&self, board: &T, moves: &mut Vec<Move>);

    /// Fills `moves` with only legal moves (king not left in check).
    fn legal(&self, board: &T, moves: &mut Vec<Move>);

    /// Captures only.
    fn captures(&self, board: &T, moves: &mut Vec<Move>);

    /// Quiet (non-capture, non-EP, non-castle) moves only.
    fn quiet_moves(&self, board: &T, moves: &mut Vec<Move>);
}
