//! Move generation: pseudo-legal and legal move enumeration, the legality
//! predicates behind it, perft, and UCI move resolution.

mod generator;
mod legality;
mod move_list;
mod perft;
mod uci_move;

pub use generator::{captures, checks, legal, quiets};
pub use legality::is_legal;
pub use move_list::{MAX_MOVES, MoveList};

pub use perft::{PerftReport, perft, perft_report};
pub use uci_move::parse_uci_move;
