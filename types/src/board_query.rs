//! Query trait describing the minimal, read-only board interface shared across crates.
//!
//! Implementors should provide inexpensive, side-effect-free accessors. Expensive
//! computations should be avoided or cached by the implementor. The trait is
//! intentionally small to reduce coupling between low-level crates such as
//! `movegen`, `perft`, and higher-level components.

use crate::{Color, Piece, Square};

/// Lightweight read-only view over a chess position used by lower-level crates.
///
/// Semantics:
/// - `piece_at`: returns the piece and color on the given square, if any.
/// - `is_square_occupied`: true if any piece occupies the square.
/// - `is_square_attacked`: whether the square is attacked by the given color under current occupancy.
/// - `get_king_square`: location of the given color's king if present.
/// - `can_castle_short`/`can_castle_long`: whether the side retains corresponding castling rights.
///   Safety of crossing squares is checked by move generation when producing castle moves.
/// - `en_passant_square`: current en-passant target square if available.
/// - `side_to_move`: whose turn it is.
pub trait BoardQuery {
    /// Piece and color at square, if any.
    fn piece_at(&self, square: Square) -> Option<(Piece, Color)>;
    /// True if any piece occupies the square.
    fn is_square_occupied(&self, square: Square) -> bool;
    /// True if the given square is attacked by pieces of `by_color` under the current position.
    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool;
    /// King square for `color`, if present.
    fn get_king_square(&self, color: Color) -> Option<Square>;
    /// Whether side can castle short (right exists); path safety is validated by consumers.
    fn can_castle_short(&self, color: Color) -> bool;
    /// Whether side can castle long (right exists); path safety is validated by consumers.
    fn can_castle_long(&self, color: Color) -> bool;
    /// En-passant target square, if any.
    fn en_passant_square(&self) -> Option<Square>;
    /// Side to move.
    fn side_to_move(&self) -> Color;
}
