use crate::Board;
use aether_core::{ALL_PIECES, Color, Piece, Square};

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
    /// Returns the Zobrist hash of the current position
    fn zobrist_hash_raw(&self) -> u64;
}

impl BoardQuery for Board {
    fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        if !self.cache.occupied.has(square) {
            return None;
        }

        let color = if self.cache.color_combined[Color::White as usize].has(square) {
            Color::White
        } else {
            Color::Black
        };

        for (i, piece_bb) in self.pieces[color as usize].iter().enumerate() {
            if piece_bb.has(square) {
                return Some((ALL_PIECES[i], color));
            }
        }

        None
    }

    fn is_square_occupied(&self, square: Square) -> bool {
        self.cache.occupied.has(square)
    }

    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        !self.attackers_to_square(square, by_color).is_empty()
    }

    fn piece_count(&self, piece: Piece, color: Color) -> u32 {
        self.pieces[color as usize][piece as usize].len()
    }

    fn get_king_square(&self, color: Color) -> Option<Square> {
        self.pieces[color as usize][Piece::King as usize].to_square()
    }

    fn can_castle_short(&self, color: Color) -> bool {
        self.game_state.castling_rights[color as usize]
            .short
            .is_some()
    }

    fn can_castle_long(&self, color: Color) -> bool {
        self.game_state.castling_rights[color as usize]
            .long
            .is_some()
    }

    fn en_passant_square(&self) -> Option<Square> {
        self.game_state.en_passant_square
    }

    fn side_to_move(&self) -> Color {
        self.game_state.side_to_move
    }

    fn zobrist_hash_raw(&self) -> u64 {
        self.zobrist_hash
    }
}
