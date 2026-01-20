use board::Board;

fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = Vec::new();
    movegen::legal(board, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0u64;
    for mv in moves {
        board.make_move(&mv).unwrap();
        nodes += perft(board, depth - 1);
        board.unmake_move(&mv).unwrap();
    }
    nodes
}

#[test]
fn test_perft_starting_position() {
    let mut board = Board::starting_position().unwrap();

    let expected = [20, 400, 8902, 197281, 4865609];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, depth);
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

#[test]
fn test_perft_kiwipete() {
    let mut board: Board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        .parse()
        .unwrap();

    let expected = [48, 2039, 97862];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, depth);
        println!(
            "Kiwipete depth {}: {} (expected {})",
            depth, count, expected_count
        );
        assert_eq!(count, expected_count, "Perft {} failed for kiwipete", depth);
    }
}

#[test]
fn test_perft_position3() {
    let mut board: Board = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".parse().unwrap();

    let expected = [14, 191, 2812, 43238, 674624];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, depth);
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

#[test]
fn test_perft_position4() {
    let mut board: Board = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"
        .parse()
        .unwrap();

    let expected = [6, 264, 9467, 422333];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, depth);
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

#[test]
fn test_perft_position5() {
    let mut board: Board = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"
        .parse()
        .unwrap();

    let expected = [44, 1486, 62379];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, depth);
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

#[test]
fn test_perft_position6() {
    let mut board: Board =
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
            .parse()
            .unwrap();

    let expected = [46, 2079, 89890];

    for (depth, &expected_count) in expected.iter().enumerate() {
        let depth = (depth + 1) as u32;
        let count = perft(&mut board, depth);
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
