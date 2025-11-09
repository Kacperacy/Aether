use board::{Board, BoardOps, FenOps};
use aether_types::MoveGen;
use movegen::Generator;

/// Perft (performance test) - counts leaf nodes at given depth
/// This is THE standard test for move generation correctness
fn perft(board: &Board, depth: usize, generator: &Generator) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = Vec::new();
    generator.legal(board, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0;
    for mv in moves {
        let mut new_board = board.clone();
        if new_board.make_move(mv).is_ok() {
            nodes += perft(&new_board, depth - 1, generator);
        }
    }

    nodes
}

#[test]
fn test_perft_startpos() {
    let board = Board::starting_position().unwrap();
    let generator = Generator::new();

    // Known correct values for starting position
    assert_eq!(perft(&board, 1, &generator), 20, "perft(1) failed");
    assert_eq!(perft(&board, 2, &generator), 400, "perft(2) failed");
    assert_eq!(perft(&board, 3, &generator), 8902, "perft(3) failed");

    // Depth 4 takes ~1 second
    println!("Testing perft(4)...");
    let result = perft(&board, 4, &generator);
    assert_eq!(result, 197281, "perft(4) failed - got {}", result);
}

#[test]
fn test_perft_position_2() {
    // Kiwipete position - tests castling, en passant, promotions
    let board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    let generator = Generator::new();

    assert_eq!(perft(&board, 1, &generator), 48, "Kiwipete perft(1) failed");
    assert_eq!(perft(&board, 2, &generator), 2039, "Kiwipete perft(2) failed");
    assert_eq!(perft(&board, 3, &generator), 97862, "Kiwipete perft(3) failed");
}

#[test]
fn test_perft_position_3() {
    // Position 3 - endgame position
    let board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    let generator = Generator::new();

    assert_eq!(perft(&board, 1, &generator), 14, "Position 3 perft(1) failed");
    assert_eq!(perft(&board, 2, &generator), 191, "Position 3 perft(2) failed");
    assert_eq!(perft(&board, 3, &generator), 2812, "Position 3 perft(3) failed");
    assert_eq!(perft(&board, 4, &generator), 43238, "Position 3 perft(4) failed");
}

#[test]
fn test_perft_position_4() {
    // Position 4 - tests discovered checks
    let board = Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1").unwrap();
    let generator = Generator::new();

    assert_eq!(perft(&board, 1, &generator), 6, "Position 4 perft(1) failed");
    assert_eq!(perft(&board, 2, &generator), 264, "Position 4 perft(2) failed");
    assert_eq!(perft(&board, 3, &generator), 9467, "Position 4 perft(3) failed");
}

#[test]
fn test_perft_position_5() {
    // Position 5 - tests en passant
    let board = Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    let generator = Generator::new();

    assert_eq!(perft(&board, 1, &generator), 44, "Position 5 perft(1) failed");
    assert_eq!(perft(&board, 2, &generator), 1486, "Position 5 perft(2) failed");
    assert_eq!(perft(&board, 3, &generator), 62379, "Position 5 perft(3) failed");
}

#[test]
fn test_perft_position_6() {
    // Position 6 - complex middle game
    let board = Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10").unwrap();
    let generator = Generator::new();

    assert_eq!(perft(&board, 1, &generator), 46, "Position 6 perft(1) failed");
    assert_eq!(perft(&board, 2, &generator), 2079, "Position 6 perft(2) failed");
    assert_eq!(perft(&board, 3, &generator), 89890, "Position 6 perft(3) failed");
}
