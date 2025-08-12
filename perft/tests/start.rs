use board::Board;
use perft::perft_count;

#[test]
fn startpos_depth1() {
    let board = Board::starting_position().expect("failed to build starting position");
    let nodes = perft_count(&board, 1);
    assert_eq!(nodes, 20);
}

#[test]
fn startpos_depth2() {
    let board = Board::starting_position().expect("failed to build starting position");
    let nodes = perft_count(&board, 2);
    assert_eq!(nodes, 400);
}

#[test]
fn startpos_depth3() {
    let board = Board::starting_position().expect("failed to build starting position");
    let nodes = perft_count(&board, 3);
    assert_eq!(nodes, 8902);
}
