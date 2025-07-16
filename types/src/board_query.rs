use crate::{Color, Piece, Square};

pub trait BoardQuery {
    fn piece_at(&self, square: Square) -> Option<(Piece, Color)>;
    fn is_square_occupied(&self, square: Square) -> bool;
    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool;
    fn get_king_square(&self, color: Color) -> Option<Square>;
    fn can_castle_short(&self, color: Color) -> bool;
    fn can_castle_long(&self, color: Color) -> bool;
    fn en_passant_square(&self) -> Option<Square>;
    fn side_to_move(&self) -> Color;
}
