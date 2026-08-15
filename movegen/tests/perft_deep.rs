//! Deep perft tier.
//!
//! These are the authoritative move-generation correctness checks, but they are
//! far too slow for the default test run — they are `#[ignore]`d and meant to be
//! run explicitly in release:
//!
//! ```bash
//! cargo test -p movegen --test perft_deep --release -- --ignored --nocapture
//! ```
//!
//! The fast tier in `perft.rs` runs on every `cargo test`.

use board::Board;
use movegen::perft;

fn check(fen: &str, depth: u32, expected: u64) {
    let mut board: Board = fen.parse().expect("valid FEN");
    let start = std::time::Instant::now();
    let nodes = perft(&mut board, depth);
    let elapsed = start.elapsed();

    let nps = if elapsed.as_millis() > 0 {
        (u128::from(nodes) * 1000 / elapsed.as_millis()) as u64
    } else {
        0
    };
    println!("depth {depth}: {nodes} nodes in {elapsed:?} ({nps} nps) — {fen}");

    assert_eq!(nodes, expected, "perft({depth}) mismatch for {fen}");
}

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
const POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
const POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
const POSITION_6: &str = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

#[test]
#[ignore = "slow: run with --release --ignored"]
fn test_perft_startpos_deep() {
    check(STARTPOS, 6, 119_060_324);
}

#[test]
#[ignore = "slow: run with --release --ignored"]
fn test_perft_startpos_very_deep() {
    check(STARTPOS, 7, 3_195_901_860);
}

#[test]
#[ignore = "slow: run with --release --ignored"]
fn test_perft_kiwipete_deep() {
    check(KIWIPETE, 5, 193_690_690);
}

#[test]
#[ignore = "slow: run with --release --ignored"]
fn test_perft_position3_deep() {
    check(POSITION_3, 7, 178_633_661);
}

#[test]
#[ignore = "slow: run with --release --ignored"]
fn test_perft_position4_deep() {
    check(POSITION_4, 6, 706_045_033);
}

#[test]
#[ignore = "slow: run with --release --ignored"]
fn test_perft_position5_deep() {
    check(POSITION_5, 5, 89_941_194);
}

#[test]
#[ignore = "slow: run with --release --ignored"]
fn test_perft_position6_deep() {
    check(POSITION_6, 6, 6_923_051_137);
}
