use board::{Board, BoardOps, FenOps};
use movegen::{Generator, MoveGen};

fn perft(board: &mut Board, generator: &Generator, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = Vec::new();
    generator.legal(board, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0u64;
    for mv in moves {
        board.make_move(&mv).unwrap();
        nodes += perft(board, generator, depth - 1);
        board.unmake_move(&mv).unwrap();
    }
    nodes
}

// Starting position tests
#[test]
fn test_perft_starting_position() {
    let mut board = Board::starting_position().unwrap();
    let generator = Generator::new();

    // Expected values from https://www.chessprogramming.org/Perft_Results
    let expected = [20, 400, 8902, 197281, 4865609];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, &generator, depth);
        println!(
            "Starting position depth {}: {} (expected {})",
            depth, count, expected_count
        );
        assert_eq!(
            count, expected_count,
            "Perft {} failed for starting position",
            depth
        );
    }
}

// Kiwipete position - complex position for testing
#[test]
fn test_perft_kiwipete() {
    let mut board =
        Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
            .unwrap();
    let generator = Generator::new();

    let expected = [48, 2039, 97862];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, &generator, depth);
        println!(
            "Kiwipete depth {}: {} (expected {})",
            depth, count, expected_count
        );
        assert_eq!(count, expected_count, "Perft {} failed for kiwipete", depth);
    }
}

// Position 3 from Chess Programming Wiki
#[test]
fn test_perft_position3() {
    let mut board = Board::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();
    let generator = Generator::new();

    let expected = [14, 191, 2812, 43238, 674624];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, &generator, depth);
        println!(
            "Position 3 depth {}: {} (expected {})",
            depth, count, expected_count
        );
        assert_eq!(
            count, expected_count,
            "Perft {} failed for position 3",
            depth
        );
    }
}

// Position 4 from Chess Programming Wiki
#[test]
fn test_perft_position4() {
    let mut board =
        Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
            .unwrap();
    let generator = Generator::new();

    let expected = [6, 264, 9467, 422333];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, &generator, depth);
        println!(
            "Position 4 depth {}: {} (expected {})",
            depth, count, expected_count
        );
        assert_eq!(
            count, expected_count,
            "Perft {} failed for position 4",
            depth
        );
    }
}

// Position 5 from Chess Programming Wiki
#[test]
fn test_perft_position5() {
    let mut board =
        Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    let generator = Generator::new();

    let expected = [44, 1486, 62379];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, &generator, depth);
        println!(
            "Position 5 depth {}: {} (expected {})",
            depth, count, expected_count
        );
        assert_eq!(
            count, expected_count,
            "Perft {} failed for position 5",
            depth
        );
    }
}

// Position 6 from Chess Programming Wiki
#[test]
fn test_perft_position6() {
    let mut board =
        Board::from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10")
            .unwrap();
    let generator = Generator::new();

    let expected = [46, 2079, 89890];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, &generator, depth);
        println!(
            "Position 6 depth {}: {} (expected {})",
            depth, count, expected_count
        );
        assert_eq!(
            count, expected_count,
            "Perft {} failed for position 6",
            depth
        );
    }
}
