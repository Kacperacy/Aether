use crate::Board;
use crate::query::BoardQuery;
use aether_types::zobrist_keys::zobrist_keys;
use aether_types::{ALL_COLORS, Color, Piece, Square};

impl Board {
    /// Calculate zobrist hash from scratch for current position
    pub fn calculate_zobrist_hash(&self) -> u64 {
        let keys = zobrist_keys();
        let mut hash = 0u64;

        // Hash all pieces on board
        for square_idx in 0..64 {
            let square = Square::from_index(square_idx);
            if let Some((piece, color)) = self.piece_at(square) {
                hash ^= keys.piece_key(square, piece, color);
            }
        }

        // Hash side to move
        if self.side_to_move() == Color::Black {
            hash ^= keys.side_to_move;
        }

        // Hash castling rights
        for color in ALL_COLORS {
            if self.can_castle_short(color) {
                hash ^= keys.castling_key(color, true);
            }
            if self.can_castle_long(color) {
                hash ^= keys.castling_key(color, false);
            }
        }

        // Hash en passant square
        if let Some(ep_square) = self.en_passant_square() {
            hash ^= keys.en_passant_key(ep_square.file());
        }

        hash
    }

    /// Update zobrist hash incrementally during move
    pub fn update_zobrist_incremental(
        &mut self,
        piece: Piece,
        color: Color,
        from: Square,
        to: Square,
    ) {
        let keys = zobrist_keys();

        // Remove piece from old square
        self.zobrist_hash ^= keys.piece_key(from, piece, color);

        // Add piece to new square
        self.zobrist_hash ^= keys.piece_key(to, piece, color);
    }

    /// Refresh zobrist hash from current position
    pub fn refresh_zobrist_hash(&mut self) {
        self.zobrist_hash = self.calculate_zobrist_hash();
    }
}
