#[cfg(test)]
mod tests {
    use aether_types::{BitBoard, File, Rank, Square};
    use movegen::MagicBitboards;

    #[test]
    fn test_magic_bitboards_initialization() {
        let magic_bitboards = MagicBitboards::new();

        // Verify all rook magics are initialized
        for square_idx in 0..64 {
            let square = Square::from_index(square_idx as i8);
            let magic = &magic_bitboards.rook_magics[square_idx];
            assert_ne!(
                magic.mask,
                BitBoard::EMPTY,
                "Rook mask for square {:?} should not be empty",
                square
            );
            assert_ne!(
                magic.magic, 0,
                "Rook magic for square {:?} should not be zero",
                square
            );
        }

        // Verify all bishop magics are initialized
        for square_idx in 0..64 {
            let square = Square::from_index(square_idx as i8);
            let magic = &magic_bitboards.bishop_magics[square_idx];
            assert_ne!(
                magic.mask,
                BitBoard::EMPTY,
                "Bishop mask for square {:?} should not be empty",
                square
            );
            assert_ne!(
                magic.magic, 0,
                "Bishop magic for square {:?} should not be zero",
                square
            );
        }
    }

    #[test]
    fn test_rook_attacks() {
        let magic_bitboards = MagicBitboards::new();
        let square = Square::E4;
        let empty_board = BitBoard::EMPTY;

        // On an empty board, rook should attack all squares in its file and rank
        let attacks = magic_bitboards.get_rook_attacks(square, empty_board);

        // Should attack all squares in the E file (excluding E4 itself)
        for rank in [
            Rank::One,
            Rank::Two,
            Rank::Three,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
        ] {
            assert!(
                attacks.has(Square::new(File::E, rank)),
                "Rook on E4 should attack {:?}",
                Square::new(File::E, rank)
            );
        }

        // Should attack all squares in the 4th rank (excluding E4 itself)
        for file in [
            File::A,
            File::B,
            File::C,
            File::D,
            File::F,
            File::G,
            File::H,
        ] {
            assert!(
                attacks.has(Square::new(file, Rank::Four)),
                "Rook on E4 should attack {:?}",
                Square::new(file, Rank::Four)
            );
        }

        // Should not attack E4 itself
        assert!(
            !attacks.has(square),
            "Rook should not attack its own square"
        );

        // Should not attack any other squares
        assert!(!attacks.has(Square::B2), "Rook on E4 should not attack B2");
    }

    #[test]
    fn test_bishop_attacks() {
        let magic_bitboards = MagicBitboards::new();
        let square = Square::E4;
        let empty_board = BitBoard::EMPTY;

        // On an empty board, bishop should attack all squares in its diagonals
        let attacks = magic_bitboards.get_bishop_attacks(square, empty_board);

        // Northeast diagonal
        assert!(attacks.has(Square::F5));
        assert!(attacks.has(Square::G6));
        assert!(attacks.has(Square::H7));

        // Northwest diagonal
        assert!(attacks.has(Square::D5));
        assert!(attacks.has(Square::C6));
        assert!(attacks.has(Square::B7));
        assert!(attacks.has(Square::A8));

        // Southeast diagonal
        assert!(attacks.has(Square::F3));
        assert!(attacks.has(Square::G2));
        assert!(attacks.has(Square::H1));

        // Southwest diagonal
        assert!(attacks.has(Square::D3));
        assert!(attacks.has(Square::C2));
        assert!(attacks.has(Square::B1));

        // Should not attack E4 itself
        assert!(!attacks.has(square));

        // Should not attack non-diagonal squares
        assert!(!attacks.has(Square::E5));
        assert!(!attacks.has(Square::F4));
    }

    #[test]
    fn test_queen_attacks() {
        let magic_bitboards = MagicBitboards::new();
        let square = Square::E4;
        let empty_board = BitBoard::EMPTY;

        // Queen attacks should be the union of rook and bishop attacks
        let queen_attacks = magic_bitboards.get_queen_attacks(square, empty_board);
        let rook_attacks = magic_bitboards.get_rook_attacks(square, empty_board);
        let bishop_attacks = magic_bitboards.get_bishop_attacks(square, empty_board);

        assert_eq!(queen_attacks, rook_attacks | bishop_attacks);
    }

    #[test]
    fn test_rook_blocked_attacks() {
        let magic_bitboards = MagicBitboards::new();
        let square = Square::E4;

        // Create a board with a blocker at E6
        let blocker_e6 = Square::E6.bitboard();

        // Rook should not attack beyond the blocker
        let attacks = magic_bitboards.get_rook_attacks(square, blocker_e6);

        // Should attack E5 and E6 (the blocker)
        assert!(attacks.has(Square::E5));
        assert!(attacks.has(Square::E6));

        // Should not attack E7 or E8 (beyond the blocker)
        assert!(!attacks.has(Square::E7));
        assert!(!attacks.has(Square::E8));
    }

    #[test]
    fn test_bishop_blocked_attacks() {
        let magic_bitboards = MagicBitboards::new();
        let square = Square::E4;

        // Create a board with a blocker at G6
        let blocker_g6 = Square::G6.bitboard();

        // Bishop should not attack beyond the blocker
        let attacks = magic_bitboards.get_bishop_attacks(square, blocker_g6);

        // Should attack F5 and G6 (the blocker)
        assert!(attacks.has(Square::F5));
        assert!(attacks.has(Square::G6));

        // Should not attack H7 (beyond the blocker)
        assert!(!attacks.has(Square::H7));
    }
}
