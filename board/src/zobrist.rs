use crate::Board;
use aether_core::zobrist_keys::zobrist_keys;
use aether_core::{ALL_COLORS, ALL_SQUARES, CastlingRights, Color, File, Piece, Square};

impl Board {
    pub fn calculate_zobrist_hash(&self) -> u64 {
        let keys = zobrist_keys();
        let mut hash = 0u64;

        for &square in &ALL_SQUARES {
            if let Some((piece, color)) = self.piece_at(square) {
                hash ^= keys.piece_key(square, piece, color);
            }
        }

        if self.side_to_move() == Color::Black {
            hash ^= keys.side_to_move;
        }

        for color in ALL_COLORS {
            if self.can_castle_short(color) {
                hash ^= keys.castling_key(color, true);
            }
            if self.can_castle_long(color) {
                hash ^= keys.castling_key(color, false);
            }
        }

        if let Some(ep_square) = self.en_passant_square() {
            hash ^= keys.en_passant_key(ep_square.file());
        }

        hash
    }

    #[inline(always)]
    pub(crate) fn zobrist_toggle_piece(&mut self, square: Square, piece: Piece, color: Color) {
        let keys = zobrist_keys();
        self.state.zobrist_hash ^= keys.piece_key(square, piece, color);
    }

    #[inline(always)]
    pub(crate) fn zobrist_toggle_side(&mut self) {
        let keys = zobrist_keys();
        self.state.zobrist_hash ^= keys.side_to_move;
    }

    #[inline(always)]
    pub(crate) fn zobrist_toggle_castling(&mut self, color: Color, kingside: bool) {
        let keys = zobrist_keys();
        self.state.zobrist_hash ^= keys.castling_key(color, kingside);
    }

    #[inline(always)]
    pub(crate) fn zobrist_toggle_en_passant(&mut self, file: File) {
        let keys = zobrist_keys();
        self.state.zobrist_hash ^= keys.en_passant_key(file);
    }

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

    #[test]
    fn test_zobrist_consistency() {
        let board: Board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
            .parse()
            .unwrap();

        let hash1 = board.calculate_zobrist_hash();
        let hash2 = board.calculate_zobrist_hash();

        assert_eq!(hash1, hash2, "Same position should have same hash");
    }

    #[test]
    fn test_zobrist_different_positions() {
        let board1: Board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
            .parse()
            .unwrap();
        let board2: Board = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
            .parse()
            .unwrap();

        let hash1 = board1.calculate_zobrist_hash();
        let hash2 = board2.calculate_zobrist_hash();

        assert_ne!(
            hash1, hash2,
            "Different positions should have different hashes"
        );
    }

    #[test]
    fn test_zobrist_side_to_move() {
        let board1: Board = "k7/8/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let board2: Board = "k7/8/8/8/8/8/8/4K3 b - - 0 1".parse().unwrap();

        let hash1 = board1.calculate_zobrist_hash();
        let hash2 = board2.calculate_zobrist_hash();

        assert_ne!(hash1, hash2, "Different side to move should change hash");
    }

    #[test]
    fn test_zobrist_toggle_piece() {
        let mut board: Board = "k7/8/8/8/8/8/8/4K3 w - - 0 1".parse().unwrap();
        let initial_hash = board.calculate_zobrist_hash();

        // Toggle piece twice should return to original hash
        board.zobrist_toggle_piece(Square::E1, Piece::King, Color::White);
        board.zobrist_toggle_piece(Square::E1, Piece::King, Color::White);

        assert_eq!(board.zobrist_hash_raw(), initial_hash);
    }

    #[test]
    fn test_incremental_vs_full_hash() {
        use aether_core::{Move, MoveFlags};

        let mut board: Board = "rn2k2r/pppppppp/8/8/8/8/PPPPPPPP/RN2K2R w KQkq - 0 1"
            .parse()
            .unwrap();

        assert_eq!(
            board.zobrist_hash_raw(),
            board.calculate_zobrist_hash(),
            "Initial hash mismatch"
        );

        let moves = [
            // 1. e4
            Move::new(Square::E2, Square::E4, Piece::Pawn).with_flags(MoveFlags {
                is_double_pawn_push: true,
                ..Default::default()
            }),
            // 1... e5
            Move::new(Square::E7, Square::E5, Piece::Pawn).with_flags(MoveFlags {
                is_double_pawn_push: true,
                ..Default::default()
            }),
            // 2. Nc3
            Move::new(Square::B1, Square::C3, Piece::Knight),
            // 2... Nc6
            Move::new(Square::B8, Square::C6, Piece::Knight),
            // 3. O-O (kingside castling)
            Move::new(Square::E1, Square::G1, Piece::King).with_flags(MoveFlags {
                is_castle: true,
                ..Default::default()
            }),
            // 3... O-O-O (queenside castling)
            Move::new(Square::E8, Square::C8, Piece::King).with_flags(MoveFlags {
                is_castle: true,
                ..Default::default()
            }),
        ];

        for (i, mv) in moves.iter().enumerate() {
            board.make_move(mv).unwrap();
            assert_eq!(
                board.zobrist_hash_raw(),
                board.calculate_zobrist_hash(),
                "Hash mismatch after move {} ({:?})",
                i + 1,
                mv
            );
        }

        for (i, mv) in moves.iter().enumerate().rev() {
            board.unmake_move(mv).unwrap();
            assert_eq!(
                board.zobrist_hash_raw(),
                board.calculate_zobrist_hash(),
                "Hash mismatch after unmake move {} ({:?})",
                i + 1,
                mv
            );
        }
    }

    #[test]
    fn test_incremental_hash_with_captures_and_promotion() {
        use aether_core::{Move, MoveFlags};

        let mut board: Board = "r3k2r/pPpppppp/8/3Pp3/8/8/P1PPPPPP/R3K2R w KQkq e6 0 1"
            .parse()
            .unwrap();

        assert_eq!(
            board.zobrist_hash_raw(),
            board.calculate_zobrist_hash(),
            "Initial hash mismatch"
        );

        let moves = [
            // 1. dxe6 (en passant capture)
            Move::new(Square::D5, Square::E6, Piece::Pawn)
                .with_capture(Piece::Pawn)
                .with_flags(MoveFlags {
                    is_en_passant: true,
                    ..Default::default()
                }),
            // 1... d6
            Move::new(Square::D7, Square::D6, Piece::Pawn),
            // 2. bxa8=Q (promotion with capture)
            Move::new(Square::B7, Square::A8, Piece::Pawn)
                .with_capture(Piece::Rook)
                .with_promotion(Piece::Queen),
        ];

        for (i, mv) in moves.iter().enumerate() {
            board.make_move(mv).unwrap();
            assert_eq!(
                board.zobrist_hash_raw(),
                board.calculate_zobrist_hash(),
                "Hash mismatch after move {} ({:?})",
                i + 1,
                mv
            );
        }

        for (i, mv) in moves.iter().enumerate().rev() {
            board.unmake_move(mv).unwrap();
            assert_eq!(
                board.zobrist_hash_raw(),
                board.calculate_zobrist_hash(),
                "Hash mismatch after unmake move {} ({:?})",
                i + 1,
                mv
            );
        }
    }
}
