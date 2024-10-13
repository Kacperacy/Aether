use aether::bitboard::Bitboard;
use aether::board::{Board, CastlingRights, Color, Piece};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_starting_position() {
        let board = Board::init();
        assert_eq!(board.turn, Color::White);
        assert!(board.white_pieces.pawns.is_set(8));
        assert!(board.black_pieces.pawns.is_set(48));
    }
    //
    // #[test]
    // fn test_square_to_index() {
    //     assert_eq!(Board::square_to_index("a1"), 0);
    //     assert_eq!(Board::square_to_index("h8"), 63);
    // }
    //
    // #[test]
    // fn test_place_piece() {
    //     let mut board = Board::new();
    //     board.place_piece(Color::White, Piece::Knight, 18);
    //     assert!(board.white_pieces.knights.is_set(18));
    // }
    //
    // #[test]
    // fn test_is_pawn_starting_position() {
    //     assert!(Board::is_pawn_starting_position(Color::White, 8));
    //     assert!(!Board::is_pawn_starting_position(Color::White, 16));
    // }
    //
    // #[test]
    // fn test_is_square_empty() {
    //     let board = Board::new();
    //     assert!(Board::is_square_empty(16, board.white_occupancy));
    // }
    //
    // #[test]
    // fn test_is_square_enemy() {
    //     let board =
    //         Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    //     assert!(board.is_square_enemy(Color::White, 48)); // a7 should have a black pawn
    //     assert!(!board.is_square_enemy(Color::White, 8)); // a2 should have a white pawn
    // }
    //
    // #[test]
    // fn test_is_check() {
    //     let board = Board::starting_position();
    //     assert_eq!(board.is_check(), None);
    // }
    //
    // #[test]
    // fn test_generate_pawn_attacks() {
    //     let board = Board::new();
    //     let white_pawns =
    //         Bitboard(0b0000000000000000000000000000000000000000000000000000000000100000);
    //     let black_pawns = Bitboard(0b100000000000000000000000000000000000000000000000000000000000);
    //
    //     let white_attacks = Board::generate_pawn_attacks(Color::White, white_pawns);
    //     let black_attacks = Board::generate_pawn_attacks(Color::Black, black_pawns);
    //
    //     assert_eq!(white_attacks, Bitboard(0b101000000000000));
    //     assert_eq!(
    //         black_attacks,
    //         Bitboard(0b10100000000000000000000000000000000000000000000000000)
    //     );
    // }
    //
    // #[test]
    // fn test_generate_knight_attacks() {
    //     let board = Board::new();
    //     let knights = Bitboard(0b0000000000000000000000000000000000000000000000000000000000000010);
    //
    //     let attacks = Board::generate_knight_attacks(knights);
    //
    //     assert_eq!(attacks, Bitboard(0b1010000100000000000));
    // }
    //
    // #[test]
    // fn test_generate_bishop_attacks() {
    //     let board = Board::new();
    //     let bishops = Bitboard(0b0000000000000000000000000000000000000000000000000000000000001000);
    //     let occupancy =
    //         Bitboard(0b0000000000000000000000000000000000000000000000000000000000000000);
    //
    //     let attacks = Board::generate_bishop_attacks(bishops, occupancy);
    //
    //     let expected_attacks = Bitboard(0b1000000001000001001000100001010000000000);
    //     assert_eq!(attacks, expected_attacks);
    // }
    //
    // #[test]
    // fn test_generate_rook_attacks() {
    //     let board = Board::new();
    //     let rooks = Bitboard(0b0000000000000000000000000000000000000000000000000000000000000001);
    //     let occupancy =
    //         Bitboard(0b0000000000000000000000000000000000000000000000000000000000000000);
    //
    //     let attacks = Board::generate_rook_attacks(rooks, occupancy);
    //
    //     assert_eq!(
    //         attacks,
    //         Bitboard(0b100000001000000010000000100000001000000010000000111111110)
    //     );
    // }
    //
    // #[test]
    // fn test_generate_queen_attacks() {
    //     let board = Board::new();
    //     let queens = Bitboard(0b0000000000000000000000000000000000000000000000000000000000001000);
    //     let occupancy =
    //         Bitboard(0b0000000000000000000000000000000000000000000000000000000000000000);
    //
    //     let attacks = Board::generate_queen_attacks(queens, occupancy);
    //
    //     assert_eq!(
    //         attacks,
    //         Bitboard(0b100000001000000010001000100001001001001010100001110011110111)
    //     );
    // }
    //
    // #[test]
    // fn test_generate_king_attacks() {
    //     let board = Board::new();
    //     let king = Bitboard(0b0000000000000000000000000000000000000000000000000000000000001000);
    //
    //     let attacks = Board::generate_king_attacks(king);
    //
    //     assert_eq!(attacks, Bitboard(0b1110000010100));
    // }
    //
    // #[test]
    // fn test_from_fen_starting_position() {
    //     let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    //     let board = Board::from_fen(fen).unwrap();
    //
    //     assert_eq!(
    //         board.white_pieces.pawns,
    //         Bitboard(0b0000000000000000000000000000000000000000000000001111111100000000)
    //     );
    //     assert_eq!(
    //         board.white_pieces.knights,
    //         Bitboard(0b0000000000000000000000000000000000000000000000000000000001000010)
    //     );
    //     assert_eq!(
    //         board.white_pieces.bishops,
    //         Bitboard(0b0000000000000000000000000000000000000000000000000000000000100100)
    //     );
    //     assert_eq!(
    //         board.white_pieces.rooks,
    //         Bitboard(0b0000000000000000000000000000000000000000000000000000000010000001)
    //     );
    //     assert_eq!(
    //         board.white_pieces.queens,
    //         Bitboard(0b0000000000000000000000000000000000000000000000000000000000001000)
    //     );
    //     assert_eq!(
    //         board.white_pieces.king,
    //         Bitboard(0b0000000000000000000000000000000000000000000000000000000000010000)
    //     );
    //
    //     assert_eq!(
    //         board.black_pieces.pawns,
    //         Bitboard(0b0000000011111111000000000000000000000000000000000000000000000000)
    //     );
    //     assert_eq!(
    //         board.black_pieces.knights,
    //         Bitboard(0b0100001000000000000000000000000000000000000000000000000000000000)
    //     );
    //     assert_eq!(
    //         board.black_pieces.bishops,
    //         Bitboard(0b0010010000000000000000000000000000000000000000000000000000000000)
    //     );
    //     assert_eq!(
    //         board.black_pieces.rooks,
    //         Bitboard(0b1000000100000000000000000000000000000000000000000000000000000000)
    //     );
    //     assert_eq!(
    //         board.black_pieces.queens,
    //         Bitboard(0b0000100000000000000000000000000000000000000000000000000000000000)
    //     );
    //     assert_eq!(
    //         board.black_pieces.king,
    //         Bitboard(0b0001000000000000000000000000000000000000000000000000000000000000)
    //     );
    //
    //     assert_eq!(board.turn, Color::White);
    //
    //     assert_eq!(
    //         board.castling_rights,
    //         CastlingRights {
    //             white_king_side: true,
    //             white_queen_side: true,
    //             black_king_side: true,
    //             black_queen_side: true,
    //         }
    //     );
    //
    //     assert_eq!(board.en_passant_square, None);
    //
    //     assert_eq!(board.halfmove_clock, 0);
    //     assert_eq!(board.fullmove_number, 1);
    // }
    //
    // #[test]
    // fn test_generate_pawn_moves() {
    //     let white_pawns = Bitboard(0b10000000000000);
    //     let black_pawns = Bitboard(0b1000000000000000000000000000000000000000000000000000);
    //
    //     let white_moves =
    //         Board::generate_pawn_moves(white_pawns, white_pawns, Bitboard(0), Color::White, None);
    //     let black_moves =
    //         Board::generate_pawn_moves(black_pawns, black_pawns, Bitboard(0), Color::Black, None);
    //
    //     assert_eq!(white_moves, vec![(13, 21), (13, 29)]);
    //     assert_eq!(black_moves, vec![(51, 43), (51, 35)]);
    // }
    //
    // #[test]
    // fn test_generate_knight_moves() {
    //     let knights = Bitboard(0b1000000000000000000000000000);
    //
    //     let moves = Board::generate_knight_moves(knights, knights, Bitboard(0));
    //
    //     assert_eq!(
    //         moves,
    //         vec![
    //             (27, 44),
    //             (27, 42),
    //             (27, 37),
    //             (27, 33),
    //             (27, 10),
    //             (27, 12),
    //             (27, 17),
    //             (27, 21)
    //         ]
    //     );
    // }
    //
    // #[test]
    // fn test_generate_bishop_moves() {
    //     let bishops = Bitboard(0b1000000000000000000000000000);
    //     let occupancy = Bitboard(0);
    //
    //     let moves = Board::generate_bishop_moves(bishops, occupancy, occupancy);
    //
    //     let expected_moves = vec![
    //         (27, 36),
    //         (27, 45),
    //         (27, 54),
    //         (27, 63),
    //         (27, 34),
    //         (27, 41),
    //         (27, 48),
    //         (27, 18),
    //         (27, 9),
    //         (27, 0),
    //         (27, 20),
    //         (27, 13),
    //         (27, 6),
    //     ];
    //     assert_eq!(moves, expected_moves);
    // }
    //
    // #[test]
    // fn test_generate_rook_moves() {
    //     let rooks = Bitboard(0b1000000000000000000000000000);
    //     let occupancy = Bitboard(0);
    //
    //     let moves = Board::generate_rook_moves(rooks, rooks, occupancy);
    //
    //     let expected_moves = vec![
    //         (27, 35),
    //         (27, 43),
    //         (27, 51),
    //         (27, 59),
    //         (27, 19),
    //         (27, 11),
    //         (27, 3),
    //         (27, 28),
    //         (27, 29),
    //         (27, 30),
    //         (27, 31),
    //         (27, 26),
    //         (27, 25),
    //         (27, 24),
    //     ];
    //     assert_eq!(moves, expected_moves);
    // }
    //
    // #[test]
    // fn test_generate_queen_moves() {
    //     let queens = Bitboard(0b1000000000000000000000000000);
    //     let occupancy = Bitboard(0);
    //
    //     let moves = Board::generate_queen_moves(queens, queens, occupancy);
    //
    //     let expected_moves = vec![
    //         (27, 36),
    //         (27, 45),
    //         (27, 54),
    //         (27, 63),
    //         (27, 34),
    //         (27, 41),
    //         (27, 48),
    //         (27, 18),
    //         (27, 9),
    //         (27, 0),
    //         (27, 20),
    //         (27, 13),
    //         (27, 6),
    //         (27, 35),
    //         (27, 43),
    //         (27, 51),
    //         (27, 59),
    //         (27, 19),
    //         (27, 11),
    //         (27, 3),
    //         (27, 28),
    //         (27, 29),
    //         (27, 30),
    //         (27, 31),
    //         (27, 26),
    //         (27, 25),
    //         (27, 24),
    //     ];
    //     assert_eq!(moves, expected_moves);
    // }
    //
    // #[test]
    // fn test_generate_king_moves() {
    //     let king = Bitboard(0b1000000000000000000000000000);
    //
    //     let moves = Board::generate_king_moves(king, king, Bitboard(0));
    //
    //     let expected_moves = vec![
    //         (27, 35),
    //         (27, 19),
    //         (27, 28),
    //         (27, 26),
    //         (27, 36),
    //         (27, 18),
    //         (27, 34),
    //         (27, 20),
    //     ];
    //     assert_eq!(moves, expected_moves);
    // }
}
