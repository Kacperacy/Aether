//! Move generation crate.
//!
//! Responsibilities:
//! - Provide fast and correct chess move generation (pseudo-legal and helpers).
//! - Encapsulate attack tables and magic bitboards.
//! - Offer a small, stable API surface for consumers (e.g., search, perft).
//!
//! This crate depends only on `aether-types` for core data structures and traits
//! (e.g., `BoardQuery`). Higher-level crates (search/engine) should depend on
//! this crate rather than duplicating attack logic.

pub mod attacks;
pub mod generator;
pub mod magic;
mod magic_constants;
#[cfg(feature = "codegen")]
pub mod magic_gen; // code generator for magic tables; see src/bin/gen_magics.rs
pub mod pieces;

use aether_types::Move;
pub use attacks::attackers_to_square_with_occ;
use board::BoardQuery;
pub use generator::Generator;

/// Public move-generation interface.
///
/// `T` is any board type that implements [`BoardQuery`] (currently `Board`).
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
