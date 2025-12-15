use crate::Board;
use crate::query::BoardQuery;
use aether_core::zobrist_keys::zobrist_keys;
use aether_core::{ALL_COLORS, CastlingRights, Color, File, Piece, Square};

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

    /// Refresh zobrist hash from current position
    pub fn refresh_zobrist_hash(&mut self) {
        self.zobrist_hash = self.calculate_zobrist_hash();
    }

    /// XOR piece key into hash (for adding or removing a piece)
    #[inline]
    pub(crate) fn zobrist_toggle_piece(&mut self, square: Square, piece: Piece, color: Color) {
        let keys = zobrist_keys();
        self.zobrist_hash ^= keys.piece_key(square, piece, color);
    }

    /// XOR side to move key into hash
    #[inline]
    pub(crate) fn zobrist_toggle_side(&mut self) {
        let keys = zobrist_keys();
        self.zobrist_hash ^= keys.side_to_move;
    }

    /// XOR castling key into hash
    #[inline]
    pub(crate) fn zobrist_toggle_castling(&mut self, color: Color, kingside: bool) {
        let keys = zobrist_keys();
        self.zobrist_hash ^= keys.castling_key(color, kingside);
    }

    /// XOR en passant key into hash
    #[inline]
    pub(crate) fn zobrist_toggle_en_passant(&mut self, file: File) {
        let keys = zobrist_keys();
        self.zobrist_hash ^= keys.en_passant_key(file);
    }

    /// Update zobrist hash for castling rights change
    pub(crate) fn zobrist_update_castling(
        &mut self,
        old_rights: &[CastlingRights; 2],
        new_rights: &[CastlingRights; 2],
    ) {
        for color in ALL_COLORS {
            let old = &old_rights[color as usize];
            let new = &new_rights[color as usize];

            // Short castling changed
            if old.short != new.short {
                self.zobrist_toggle_castling(color, true);
            }

            // Long castling changed
            if old.long != new.long {
                self.zobrist_toggle_castling(color, false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FenOps;

    #[test]
    fn test_zobrist_consistency() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        let hash1 = board.calculate_zobrist_hash();
        let hash2 = board.calculate_zobrist_hash();

        assert_eq!(hash1, hash2, "Same position should have same hash");
    }

    #[test]
    fn test_zobrist_different_positions() {
        let board1 =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let board2 =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1").unwrap();

        let hash1 = board1.calculate_zobrist_hash();
        let hash2 = board2.calculate_zobrist_hash();

        assert_ne!(
            hash1, hash2,
            "Different positions should have different hashes"
        );
    }

    #[test]
    fn test_zobrist_side_to_move() {
        let board1 = Board::from_fen("k7/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let board2 = Board::from_fen("k7/8/8/8/8/8/8/4K3 b - - 0 1").unwrap();

        let hash1 = board1.calculate_zobrist_hash();
        let hash2 = board2.calculate_zobrist_hash();

        assert_ne!(hash1, hash2, "Different side to move should change hash");
    }

    #[test]
    fn test_zobrist_toggle_piece() {
        let mut board = Board::from_fen("k7/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let initial_hash = board.calculate_zobrist_hash();

        // Toggle piece twice should return to original hash
        board.zobrist_toggle_piece(Square::E1, Piece::King, Color::White);
        board.zobrist_toggle_piece(Square::E1, Piece::King, Color::White);

        assert_eq!(board.zobrist_hash_raw(), initial_hash);
    }
}
