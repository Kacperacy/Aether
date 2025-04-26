pub mod king;
pub mod knight;
pub mod pawn;
mod utils;

pub use king::get_king_moves;
pub use knight::get_knight_moves;
pub use pawn::{get_pawn_attacks, get_pawn_moves};
