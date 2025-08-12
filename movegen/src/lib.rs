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

pub use attacks::attackers_to_square_with_occ;
pub use generator::Generator;
