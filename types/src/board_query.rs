//! Query trait describing the minimal, read-only board interface shared across crates.

use crate::{Color, Piece, Square};

/// Lightweight read-only view over a chess position used by lower-level crates.
pub trait BoardQuery {
    /// Piece and color at square, if any.
    fn piece_at(&self, square: Square) -> Option<(Piece, Color)>;
    /// True if any piece occupies the square.
    fn is_square_occupied(&self, square: Square) -> bool;
    /// True if the given square is attacked by pieces of `by_color` under the current position.
    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool;
    /// Count of `piece` for `color`.
    fn piece_count(&self, piece: Piece, color: Color) -> u32;
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
