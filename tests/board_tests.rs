use aether::bitboard::Bitboard;
use aether::board::{Board, CastlingRights, Color};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_fen_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_fen(fen).unwrap();

        assert_eq!(
            board.white_pieces.pawns,
            Bitboard(0b0000000000000000000000000000000000000000000000001111111100000000)
        );
        assert_eq!(
            board.white_pieces.knights,
            Bitboard(0b0000000000000000000000000000000000000000000000000000000001000010)
        );
        assert_eq!(
            board.white_pieces.bishops,
            Bitboard(0b0000000000000000000000000000000000000000000000000000000000100100)
        );
        assert_eq!(
            board.white_pieces.rooks,
            Bitboard(0b0000000000000000000000000000000000000000000000000000000010000001)
        );
        assert_eq!(
            board.white_pieces.queens,
            Bitboard(0b0000000000000000000000000000000000000000000000000000000000001000)
        );
        assert_eq!(
            board.white_pieces.king,
            Bitboard(0b0000000000000000000000000000000000000000000000000000000000010000)
        );

        assert_eq!(
            board.black_pieces.pawns,
            Bitboard(0b0000000011111111000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.black_pieces.knights,
            Bitboard(0b0100001000000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.black_pieces.bishops,
            Bitboard(0b0010010000000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.black_pieces.rooks,
            Bitboard(0b1000000100000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.black_pieces.queens,
            Bitboard(0b0000100000000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.black_pieces.king,
            Bitboard(0b0001000000000000000000000000000000000000000000000000000000000000)
        );

        assert_eq!(board.turn, Color::White);

        assert_eq!(
            board.castling_rights,
            CastlingRights {
                white_king_side: true,
                white_queen_side: true,
                black_king_side: true,
                black_queen_side: true,
            }
        );

        assert_eq!(board.en_passant_square, None);

        assert_eq!(board.halfmove_clock, 0);
        assert_eq!(board.fullmove_number, 1);
    }
}
