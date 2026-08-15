use aether_core::{File, Move, Rank, Square};
use board::Board;

#[test]
fn test_make_unmake_symmetry_starting_position() {
    let mut board = Board::starting_position().unwrap();
    let original_fen = board.to_string();
    let original_zobrist = board.zobrist_hash_raw();

    let mut moves = movegen::MoveList::new();
    movegen::legal(&board, &mut moves);

    for mv in moves.iter() {
        board.make_move(mv).unwrap();
        board.unmake_move(mv).unwrap();

        assert_eq!(
            board.to_string(),
            original_fen,
            "Position changed after make/unmake for move: {}",
            mv
        );

        assert_eq!(
            board.zobrist_hash_raw(),
            original_zobrist,
            "Zobrist hash changed after make/unmake for move: {}",
            mv
        );
    }
}

#[test]
fn test_make_unmake_complex_position() {
    let mut board: Board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        .parse()
        .unwrap();
    let original_fen = board.to_string();
    let original_zobrist = board.zobrist_hash_raw();

    let mut moves = movegen::MoveList::new();
    movegen::legal(&board, &mut moves);

    for mv in moves.iter() {
        board.make_move(mv).unwrap();
        board.unmake_move(mv).unwrap();

        assert_eq!(board.to_string(), original_fen);
        assert_eq!(board.zobrist_hash_raw(), original_zobrist);
    }
}

#[test]
fn test_make_unmake_castling() {
    let mut board: Board = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1"
        .parse()
        .unwrap();
    let original_fen = board.to_string();

    let mut moves = movegen::MoveList::new();
    movegen::legal(&board, &mut moves);

    let castle_move = moves
        .iter()
        .find(|mv| mv.is_castling())
        .expect("No castling move found");

    board.make_move(castle_move).unwrap();
    board.unmake_move(castle_move).unwrap();

    assert_eq!(board.to_string(), original_fen);
}

#[test]
fn test_make_unmake_en_passant() {
    let mut board: Board = "rnbqkbnr/pppp1ppp/8/3Pp3/8/8/PPP1PPPP/RNBQKBNR w KQkq e6 0 1"
        .parse()
        .unwrap();
    let original_fen = board.to_string();

    let mut moves = movegen::MoveList::new();
    movegen::legal(&board, &mut moves);

    let ep_move = moves
        .iter()
        .find(|mv| mv.is_en_passant())
        .expect("No en passant move found");

    board.make_move(ep_move).unwrap();
    board.unmake_move(ep_move).unwrap();

    assert_eq!(board.to_string(), original_fen);
}

#[test]
fn test_make_unmake_promotion() {
    let mut board: Board = "7k/P7/8/8/8/8/7p/K7 w - - 0 1".parse().unwrap();
    let original_fen = board.to_string();

    let mut moves = movegen::MoveList::new();
    movegen::legal(&board, &mut moves);

    let promo_move = moves
        .iter()
        .find(|mv| mv.is_promotion())
        .expect("No promotion move found");

    board.make_move(promo_move).unwrap();
    board.unmake_move(promo_move).unwrap();

    assert_eq!(board.to_string(), original_fen);
}

#[test]
fn test_halfmove_clock() {
    let mut board = Board::starting_position().unwrap();

    let e2 = Square::new(File::E, Rank::TWO);
    let e4 = Square::new(File::E, Rank::FOUR);
    let mv = Move::new(e2, e4, Move::DOUBLE_PUSH);

    board.make_move(&mv).unwrap();
    assert_eq!(
        board.halfmove_clock(),
        0,
        "Pawn move should reset halfmove clock"
    );
}

/// Regression for the unmake-stack invariant.
///
/// `state_history` is a fixed 256-entry ring indexed by `history_index % 256`,
/// and a replayed game shares it with search. A long game therefore used to push
/// `history_index` past the ring size; the overwritten slots happened to be
/// game-history entries that nothing unmakes back to, so it never corrupted a
/// real search — but it silently broke the buffer's invariant, and any future
/// deep unwind would have read clobbered state.
///
/// `commit_history` drops the permanently-played moves from the stack. With the
/// `debug_assert!` in `make_move`, this test panics without that call.
#[test]
fn test_long_game_then_search_respects_unmake_stack() {
    let mut board = Board::starting_position().unwrap();

    // 4-ply cycle back to the same position; 80 cycles = 320 plies > the 256 ring.
    let shuffle = [
        Move::new(Square::G1, Square::F3, Move::QUIET),
        Move::new(Square::G8, Square::F6, Move::QUIET),
        Move::new(Square::F3, Square::G1, Move::QUIET),
        Move::new(Square::F6, Square::G8, Move::QUIET),
    ];

    for cycle in 0..80 {
        for mv in &shuffle {
            board.make_move(mv).unwrap();
        }
        // Mirrors what the UCI `position` handler does once the game is replayed.
        board.commit_history();
        assert_eq!(
            board.ply(),
            (cycle + 1) * 4,
            "game history must still be tracked"
        );
    }

    // Repetition detection reads zobrist_history, which commit_history leaves alone.
    assert_eq!(board.ply(), 320);

    // Search-style make/unmake from here must be exact.
    let fen_before = board.to_string();
    let zobrist_before = board.zobrist_hash_raw();

    let mut moves = movegen::MoveList::new();
    movegen::legal(&board, &mut moves);
    assert!(!moves.is_empty());

    for mv in &moves {
        board.make_move(mv).unwrap();
        board.unmake_move(mv).unwrap();

        assert_eq!(board.to_string(), fen_before, "position corrupted: {mv}");
        assert_eq!(
            board.zobrist_hash_raw(),
            zobrist_before,
            "zobrist corrupted: {mv}"
        );
    }
}

/// The incrementally-maintained zobrist hash must equal a full recompute at
/// every node of a real tree, and must be restored exactly by `unmake_move`.
///
/// This is the invariant that matters and it does not depend on the key values,
/// so it survives changing how the keys are generated. A drift here corrupts
/// transposition-table lookups and repetition detection at once.
#[test]
fn test_incremental_zobrist_matches_full_recompute() {
    const POSITIONS: &[&str] = &[
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        // Castling rights on both sides, dense tactics.
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        // En passant available.
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        // Promotions, including capture-promotions.
        "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1",
    ];

    fn walk(board: &mut Board, depth: u32) {
        assert_eq!(
            board.zobrist_hash_raw(),
            board.calculate_zobrist_hash(),
            "incremental hash drifted from the recomputed hash"
        );

        if depth == 0 {
            return;
        }

        let mut moves = movegen::MoveList::new();
        movegen::legal(board, &mut moves);

        for mv in &moves {
            let before = board.zobrist_hash_raw();

            board.make_move(mv).unwrap();
            walk(board, depth - 1);
            board.unmake_move(mv).unwrap();

            assert_eq!(
                board.zobrist_hash_raw(),
                before,
                "unmake_move did not restore the hash after {mv}"
            );
        }
    }

    for fen in POSITIONS {
        let mut board: Board = fen.parse().expect("valid FEN");
        walk(&mut board, 3);
    }
}
