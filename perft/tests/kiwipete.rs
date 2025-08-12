use board::{Board, FenOps};
use perft::perft_count;

// Kiwipete position from CPW
// FEN: r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1
// Expected perft nodes: d1=48, d2=2039, d3=97862, d4=4085603, d5=193690690

const KIWI_FEN: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

#[test]
fn kiwipete_depth1() {
    let board = Board::from_fen(KIWI_FEN).expect("failed to parse Kiwipete fen");
    let nodes = perft_count(&board, 1);
    assert_eq!(nodes, 48);
}

#[test]
fn kiwipete_depth2() {
    let board = Board::from_fen(KIWI_FEN).expect("failed to parse Kiwipete fen");
    let nodes = perft_count(&board, 2);
    assert_eq!(nodes, 2039);
}

#[test]
fn kiwipete_depth3() {
    let board = Board::from_fen(KIWI_FEN).expect("failed to parse Kiwipete fen");
    let nodes = perft_count(&board, 3);
    assert_eq!(nodes, 97862);
}
