//! Move generation: pseudo-legal and legal move enumeration, the legality
//! predicates behind it, perft, and UCI move resolution.

mod generator;
mod legality;
mod perft;
mod uci_move;

pub use generator::{captures, checks, legal, pseudo_legal, quiet_moves};
pub use legality::would_leave_king_in_check;
pub use perft::{PerftReport, perft, perft_divide, perft_report};
pub use uci_move::parse_uci_move;
