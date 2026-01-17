use aether_core::{File, Move, Rank, Square};
use board::{Board, FenOps};
use movegen::{Generator, MoveGen};

#[test]
fn test_make_unmake_symmetry_starting_position() {
    let mut board = Board::starting_position().unwrap();
    let original_fen = board.to_fen();
    let original_zobrist = board.zobrist_hash();

    let generator = Generator::new();
    let mut moves = Vec::new();
    generator.legal(&board, &mut moves);

    for mv in moves {
        board.make_move(&mv).unwrap();
        board.unmake_move(&mv).unwrap();

        assert_eq!(
            board.to_fen(),
            original_fen,
            "Position changed after make/unmake for move: {}",
            mv
        );

        assert_eq!(
            board.zobrist_hash(),
            original_zobrist,
            "Zobrist hash changed after make/unmake for move: {}",
            mv
        );
    }
}

#[test]
fn test_make_unmake_complex_position() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut board = Board::from_fen(fen).unwrap();
    let original_fen = board.to_fen();
    let original_zobrist = board.zobrist_hash();

    let generator = Generator::new();
    let mut moves = Vec::new();
    generator.legal(&board, &mut moves);

    for mv in moves {
        board.make_move(&mv).unwrap();
        board.unmake_move(&mv).unwrap();

        assert_eq!(board.to_fen(), original_fen);
        assert_eq!(board.zobrist_hash(), original_zobrist);
    }
}

#[test]
fn test_make_unmake_castling() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let mut board = Board::from_fen(fen).unwrap();
    let original_fen = board.to_fen();

    let generator = Generator::new();
    let mut moves = Vec::new();
    generator.legal(&board, &mut moves);

    let castle_move = moves
        .iter()
        .find(|mv| mv.flags.is_castle)
        .expect("No castling move found");

    board.make_move(castle_move).unwrap();
    board.unmake_move(castle_move).unwrap();

    assert_eq!(board.to_fen(), original_fen);
}

#[test]
fn test_make_unmake_en_passant() {
    let fen = "rnbqkbnr/pppp1ppp/8/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 1";
    let mut board = Board::from_fen(fen).unwrap();
    let original_fen = board.to_fen();

    let generator = Generator::new();
    let mut moves = Vec::new();
    generator.legal(&board, &mut moves);

    let ep_move = moves
        .iter()
        .find(|mv| mv.flags.is_en_passant)
        .expect("No en passant move found");

    board.make_move(ep_move).unwrap();
    board.unmake_move(ep_move).unwrap();

    assert_eq!(board.to_fen(), original_fen);
}

#[test]
fn test_make_unmake_promotion() {
    let fen = "7k/P7/8/8/8/8/7p/K7 w - - 0 1";
    let mut board = Board::from_fen(fen).unwrap();
    let original_fen = board.to_fen();

    let generator = Generator::new();
    let mut moves = Vec::new();
    generator.legal(&board, &mut moves);

    let promo_move = moves
        .iter()
        .find(|mv| mv.promotion.is_some())
        .expect("No promotion move found");

    board.make_move(promo_move).unwrap();
    board.unmake_move(promo_move).unwrap();

    assert_eq!(board.to_fen(), original_fen);
}

#[test]
fn test_halfmove_clock() {
    let mut board = Board::starting_position().unwrap();

    // e2e4 (pawn move - reset clock)
    let e2 = Square::new(File::E, Rank::Two);
    let e4 = Square::new(File::E, Rank::Four);
    let mv = Move::new(e2, e4, aether_core::Piece::Pawn).with_flags(aether_core::MoveFlags {
        is_double_pawn_push: true,
        ..Default::default()
    });

    board.make_move(&mv).unwrap();
    assert_eq!(
        board.halfmove_clock(),
        0,
        "Pawn move should reset halfmove clock"
    );
}
