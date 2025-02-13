use aether::bitboard::Bitboard;
use aether::board::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_init() {
        let board = Board::init();
        assert_eq!(board.turn, Color::White);
        assert!(board.pieces[Piece::Pawn as usize]
            .and(&board.colors[Color::Black as usize])
            .is_set(8));
        assert!(board.pieces[Color::Black as usize]
            .and(&board.colors[Color::Black as usize])
            .is_set(48));
    }

    #[test]
    fn test_board_reset() {
        let mut board = Board::new();
        board.reset();
        assert_eq!(board.turn, Color::White);
        assert_eq!(board.game_state.fifty_move_ply_count, 0);
        assert_eq!(board.ply, 0);
        assert!(board.colors[Color::White as usize].is_set(0));
        assert!(board.colors[Color::Black as usize].is_set(0));
        board.pieces.iter().for_each(|pieces| {
            assert_eq!(pieces.value(), 0);
        });
    }

    #[test]
    fn test_board_set_fen() {
        let mut board = Board::new();
        board.set_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(board.turn, Color::White);
        assert_eq!(board.ply, 0);
        assert!(board.pieces[Piece::Pawn as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(8));
        assert!(board.pieces[Piece::Pawn as usize]
            .and(&board.colors[Color::Black as usize])
            .is_set(48));
    }

    #[test]
    fn test_board_to_fen() {
        let mut board = Board::new();
        board.set_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assert_eq!(
            board.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn test_board_add_piece() {
        let mut board = Board::new();
        board.add_piece(Color::White, Piece::Knight, 1);
        assert!(board.pieces[Piece::Knight as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(1));
    }

    #[test]
    fn test_board_remove_piece() {
        let mut board = Board::new();
        board.add_piece(Color::White, Piece::Knight, 1);
        board.remove_piece(Color::White, Piece::Knight, 1);
        assert!(!board.pieces[Piece::Knight as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(1));
    }

    #[test]
    fn test_board_move_piece() {
        let mut board = Board::new();
        board.add_piece(Color::White, Piece::Knight, 1);
        board.move_piece(Color::White, Piece::Knight, 1, 18);
        assert!(!board.pieces[Piece::Knight as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(1));
        assert!(board.pieces[Piece::Knight as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(18));
    }

    #[test]
    fn test_board_make_move() {
        let mut board = Board::init();
        let mv = Move {
            from: 12,
            to: 28,
            piece: Piece::Pawn,
            color: Color::White,
            en_passant: false,
            castling: false,
            promotion: None,
            capture: None,
        };
        board.make_move(&mv);
        assert!(board.pieces[Piece::Pawn as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(28));
        assert!(!board.pieces[Piece::Pawn as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(12));
    }

    #[test]
    fn test_square_to_index() {
        assert_eq!(Board::square_to_index("a1"), 0);
        assert_eq!(Board::square_to_index("h8"), 63);
    }

    // #[test]
    // fn test_is_check() {
    //     let board = Board::starting_position();
    //     assert_eq!(board.is_check(), None);
    // }

    // #[test]
    // fn test_generate_pawn_attacks() {
    //     let mut board = Board::new();
    //     board.set_fen("8/8/8/8/8/4P3/8/8 w - - 0 1");
    //
    //     let attacks = board.generate_pawn_attacks();
    //
    //     board.set_fen("k7/8/4p3/2Pp1P2/8/8/8/KQ6 w - d6 0 2");
    //
    //     let en_passant_attacks = board.generate_pawn_attacks();
    //
    //     assert_eq!(attacks, Bitboard(671088640));
    //     assert_eq!(en_passant_attacks, Bitboard(98956046499840));
    // }
    //
    // #[test]
    // fn test_generate_knight_attacks() {
    //     let mut board = Board::new();
    //     board.set_fen("8/8/8/8/8/4N2N/8/8 w - - 0 1");
    //
    //     let attacks = board.generate_knight_attacks();
    //
    //     assert_eq!(attacks, Bitboard(448354346088));
    // }
    //
    // #[test]
    // fn test_generate_bishop_attacks() {
    //     let mut board = Board::new();
    //     board.set_fen("8/8/8/8/8/4B3/8/8 w - - 0 1");
    //
    //     let attacks = board.generate_bishop_attacks();
    //
    //     assert_eq!(attacks, Bitboard(424704217196612));
    // }
    //
    // #[test]
    // fn test_generate_rook_attacks() {
    //     let mut board = Board::new();
    //     board.set_fen("8/8/8/8/8/4R3/8/8 w - - 0 1");
    //
    //     let attacks = board.generate_rook_attacks();
    //
    //     assert_eq!(attacks, Bitboard(1157442765423841296));
    // }
    //
    // #[test]
    // fn test_generate_queen_attacks() {
    //     let mut board = Board::new();
    //     board.set_fen("8/8/8/8/8/4Q3/8/8 w - - 0 1");
    //
    //     let attacks = board.generate_queen_attacks();
    //
    //     assert_eq!(attacks, Bitboard(1157867469641037908));
    // }
    //
    // #[test]
    // fn test_generate_king_attacks() {
    //     let mut board = Board::new();
    //     board.set_fen("8/8/8/8/8/4K3/8/8 w - - 0 1");
    //
    //     let attacks = board.generate_king_attacks();
    //
    //     assert_eq!(attacks, Bitboard(942159872));
    // }

    #[test]
    fn test_from_fen_starting_position() {
        let board = Board::init();

        assert_eq!(
            board.pieces[Piece::Pawn as usize].and(&board.colors[Color::White as usize]),
            Bitboard(0b0000000000000000000000000000000000000000000000001111111100000000)
        );
        assert_eq!(
            board.pieces[Piece::Knight as usize].and(&board.colors[Color::White as usize]),
            Bitboard(0b0000000000000000000000000000000000000000000000000000000001000010)
        );
        assert_eq!(
            board.pieces[Piece::Bishop as usize].and(&board.colors[Color::White as usize]),
            Bitboard(0b0000000000000000000000000000000000000000000000000000000000100100)
        );
        assert_eq!(
            board.pieces[Piece::Rook as usize].and(&board.colors[Color::White as usize]),
            Bitboard(0b0000000000000000000000000000000000000000000000000000000010000001)
        );
        assert_eq!(
            board.pieces[Piece::Queen as usize].and(&board.colors[Color::White as usize]),
            Bitboard(0b0000000000000000000000000000000000000000000000000000000000001000)
        );
        assert_eq!(
            board.pieces[Piece::King as usize].and(&board.colors[Color::White as usize]),
            Bitboard(0b0000000000000000000000000000000000000000000000000000000000010000)
        );

        assert_eq!(
            board.pieces[Piece::Pawn as usize].and(&board.colors[Color::Black as usize]),
            Bitboard(0b0000000011111111000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.pieces[Piece::Knight as usize].and(&board.colors[Color::Black as usize]),
            Bitboard(0b0100001000000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.pieces[Piece::Bishop as usize].and(&board.colors[Color::Black as usize]),
            Bitboard(0b0010010000000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.pieces[Piece::Rook as usize].and(&board.colors[Color::Black as usize]),
            Bitboard(0b1000000100000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.pieces[Piece::Queen as usize].and(&board.colors[Color::Black as usize]),
            Bitboard(0b0000100000000000000000000000000000000000000000000000000000000000)
        );
        assert_eq!(
            board.pieces[Piece::King as usize].and(&board.colors[Color::Black as usize]),
            Bitboard(0b0001000000000000000000000000000000000000000000000000000000000000)
        );

        assert_eq!(board.turn, Color::White);

        assert_eq!(board.game_state.castling_rights, 0b1111);

        assert_eq!(board.game_state.en_passant_square, None);

        assert_eq!(board.game_state.fifty_move_ply_count, 0);
        assert_eq!(board.ply, 0);
    }

    #[test]
    fn test_generate_pawn_moves() {
        let mut board = Board::new();
        board.set_fen("k7/8/8/3Pp3/7r/6P1/P7/K7 w - e6 0 1");

        let mut moves = board.generate_pawn_moves();
        let mut moves_assert = vec![
            // DOUBLE PUSH
            Move {
                from: 8,
                to: 16,
                piece: Piece::Pawn,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 8,
                to: 24,
                piece: Piece::Pawn,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            // SINGLE PUSH
            Move {
                from: 22,
                to: 30,
                piece: Piece::Pawn,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            // CAPTURE
            Move {
                from: 22,
                to: 31,
                piece: Piece::Pawn,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: Some(Piece::Rook),
            },
            Move {
                from: 35,
                to: 43,
                piece: Piece::Pawn,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            // EN PASSANT
            Move {
                from: 35,
                to: 44,
                piece: Piece::Pawn,
                color: Color::White,
                en_passant: true,
                castling: false,
                promotion: None,
                capture: Some(Piece::Pawn),
            },
        ];

        moves.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
        moves_assert.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        assert_eq!(moves, moves_assert);
    }

    #[test]
    fn test_generate_knight_moves() {
        let mut board = Board::new();
        board.set_fen("k7/8/8/4p2N/5r2/6P1/P7/K7 w - - 0 1");

        let mut moves = board.generate_knight_moves();
        let mut moves_assert = vec![
            Move {
                from: 39,
                to: 54,
                piece: Piece::Knight,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 39,
                to: 45,
                piece: Piece::Knight,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 39,
                to: 29,
                piece: Piece::Knight,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: Some(Piece::Rook),
            },
        ];

        moves.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
        moves_assert.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        assert_eq!(moves, moves_assert);
    }

    #[test]
    fn test_generate_bishop_moves() {
        let mut board = Board::new();
        board.set_fen("k7/4P3/8/4p1B1/5r2/8/P7/K7 w - - 0 1");

        let mut moves = board.generate_bishop_moves();
        let mut moves_assert = vec![
            Move {
                from: 38,
                to: 29,
                piece: Piece::Bishop,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: Some(Piece::Rook),
            },
            Move {
                from: 38,
                to: 47,
                piece: Piece::Bishop,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 38,
                to: 31,
                piece: Piece::Bishop,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 38,
                to: 45,
                piece: Piece::Bishop,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
        ];

        moves.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
        moves_assert.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        assert_eq!(moves, moves_assert);
    }

    #[test]
    fn test_generate_rook_moves() {
        let mut board = Board::new();
        board.set_fen("k7/6P1/8/4p1R1/5r2/8/P7/K7 w - - 0 1");

        let mut moves = board.generate_rook_moves();
        let mut moves_assert = vec![
            Move {
                from: 38,
                to: 37,
                piece: Piece::Rook,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 38,
                to: 36,
                piece: Piece::Rook,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: Some(Piece::Pawn),
            },
            Move {
                from: 38,
                to: 39,
                piece: Piece::Rook,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 38,
                to: 46,
                piece: Piece::Rook,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 38,
                to: 30,
                piece: Piece::Rook,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 38,
                to: 22,
                piece: Piece::Rook,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 38,
                to: 14,
                piece: Piece::Rook,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 38,
                to: 6,
                piece: Piece::Rook,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
        ];

        moves.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
        moves_assert.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        assert_eq!(moves, moves_assert);
    }

    #[test]
    fn test_generate_queen_moves() {
        let mut board = Board::new();
        board.set_fen("k7/8/5P2/3p4/8/3r1Q2/P7/K7 w - - 0 1");

        let mut moves = board.generate_queen_moves();
        let mut moves_assert = vec![
            Move {
                from: 21,
                to: 13,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 5,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 12,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 3,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 20,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 19,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: Some(Piece::Rook),
            },
            Move {
                from: 21,
                to: 28,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 35,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: Some(Piece::Pawn),
            },
            Move {
                from: 21,
                to: 29,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 37,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 30,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 39,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 22,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 23,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 14,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 21,
                to: 7,
                piece: Piece::Queen,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
        ];

        moves.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
        moves_assert.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        assert_eq!(moves, moves_assert);
    }

    #[test]
    fn test_generate_king_moves() {
        let mut board = Board::new();
        board.set_fen("k7/8/8/2p5/8/8/P6P/6rK w - - 0 1");

        let mut moves = board.generate_king_moves();
        let mut moves_assert = vec![Move {
            from: 7,
            to: 6,
            piece: Piece::King,
            color: Color::White,
            en_passant: false,
            castling: false,
            promotion: None,
            capture: Some(Piece::Rook),
        }];

        moves.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
        moves_assert.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        // TODO: uncomment when check detection is implemented
        // assert_eq!(moves, moves_assert);
    }

    #[test]
    fn test_white_castling() {
        let mut board = Board::new();
        board.set_fen("r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/3P1N2/PPP2PPP/RNBQK2R w KQkq - 1 5");

        let mut castling = board.generate_king_moves();
        let mut castling_assert = vec![
            Move {
                from: 4,
                to: 6,
                piece: Piece::King,
                color: Color::White,
                en_passant: false,
                castling: true,
                promotion: None,
                capture: None,
            },
            Move {
                from: 4,
                to: 12,
                piece: Piece::King,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 4,
                to: 11,
                piece: Piece::King,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
            Move {
                from: 4,
                to: 5,
                piece: Piece::King,
                color: Color::White,
                en_passant: false,
                castling: false,
                promotion: None,
                capture: None,
            },
        ];
        castling.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
        castling_assert.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));

        assert_eq!(castling, castling_assert);
    }

    #[test]
    fn test_make_move() {
        let mut board = Board::init();
        let mv = Move {
            from: 12,
            to: 28,
            piece: Piece::Pawn,
            color: Color::White,
            en_passant: false,
            castling: false,
            promotion: None,
            capture: None,
        };

        board.make_move(&mv);

        assert!(board.pieces[Piece::Pawn as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(28));
        assert!(!board.pieces[Piece::Pawn as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(12));
    }

    #[test]
    fn test_undo_move() {
        let mut board = Board::init();
        let fen_before = board.to_fen();
        let mv = Move {
            from: 12,
            to: 28,
            piece: Piece::Pawn,
            color: Color::White,
            en_passant: false,
            castling: false,
            promotion: None,
            capture: None,
        };

        board.make_move(&mv);
        board.undo_move(&mv);

        assert!(board.pieces[Piece::Pawn as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(12));
        assert!(!board.pieces[Piece::Pawn as usize]
            .and(&board.colors[Color::White as usize])
            .is_set(28));
        assert_eq!(fen_before, board.to_fen());
    }
}
