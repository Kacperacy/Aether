use board::{Board, FenOps};
use perft::perft_count;

// CPW Position 3 (rook+pawns endgame)
// FEN: 8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -
// Expected nodes: d1=14, d2=191, d3=2812, d4=43238, d5=674624, d6=11030083

const POS3_FEN: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";

#[test]
fn pos3_depth1() {
    let board = Board::from_fen(POS3_FEN).expect("failed to parse pos3 fen");
    let nodes = perft_count(&board, 1);
    assert_eq!(nodes, 14);
}

#[test]
fn pos3_depth2() {
    let board = Board::from_fen(POS3_FEN).expect("failed to parse pos3 fen");
    let nodes = perft_count(&board, 2);
    assert_eq!(nodes, 191);
}

#[test]
fn pos3_depth3() {
    let board = Board::from_fen(POS3_FEN).expect("failed to parse pos3 fen");
    let nodes = perft_count(&board, 3);
    assert_eq!(nodes, 2812);
}

#[test]
fn pos3_depth4() {
    let board = Board::from_fen(POS3_FEN).expect("failed to parse pos3 fen");
    let nodes = perft_count(&board, 4);
    assert_eq!(nodes, 43238);
}
