//! `is_legal` must agree with `legal()` on *every* move encoding.
//!
//! This is the one predicate in the crate that takes untrusted input: the move
//! picker yields the transposition-table move before generating anything, so
//! nothing else can vouch for it, and a TT key collision can hand us the encoded
//! bits of a move from a completely different position.
//!
//! A `Move` is a `u16`, so "every move encoding" is only 65536 values. The test
//! is therefore exhaustive rather than sampled: for each position it checks all
//! of them against the generator's own answer. There is no gap for a hand-picked
//! case to fall through.

use aether_core::Move;
use board::Board;
use std::collections::HashSet;

/// Positions chosen to exercise every flag: castling both ways, en passant,
/// promotions with and without capture, pins, double check, and a position with
/// no castling rights at all.
const POSITIONS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    // Promotions, including capture-promotions on both wings.
    "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1",
    "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    // En passant available, and an en passant that is illegal through a pin.
    "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
    "8/8/8/K2pP2r/8/8/8/7k w - d6 0 1",
    // In check, and double check (king moves only).
    "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
    "rnb1k1nr/pppp1ppp/8/2b1p3/2B1P2q/5P2/PPPP2PP/RNBQK1NR w KQkq - 0 1",
    // Castling rights present but the path is attacked / occupied.
    "r3k2r/8/8/8/8/8/6q1/R3K2R w KQkq - 0 1",
    "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1",
    // Sparse endgame — most encodings are nonsense here.
    "8/8/4k3/8/8/4K3/4P3/8 w - - 0 1",
];

fn legal_set(board: &Board) -> HashSet<Move> {
    let mut moves = movegen::MoveList::new();
    movegen::legal(board, &mut moves);
    moves.iter().copied().collect()
}

/// The whole contract, checked over the entire encoding space.
#[test]
fn test_is_legal_agrees_with_generator_on_every_encoding() {
    for fen in POSITIONS {
        let board: Board = fen.parse().expect("valid FEN");
        let legal = legal_set(&board);

        for bits in 0..=u16::MAX {
            let mv = Move(bits);
            let expected = legal.contains(&mv);
            let actual = movegen::is_legal(&board, &mv);

            assert_eq!(
                actual,
                expected,
                "is_legal({mv}) [bits={bits:#06x}, flags={}] returned {actual}, \
                 generator says {expected}, in {fen}",
                mv.flags()
            );
        }
    }
}

/// A move that is legal in one position is almost never legal in another. This
/// is the TT-collision scenario stated plainly.
#[test]
fn test_moves_from_other_positions_are_rejected() {
    let boards: Vec<Board> = POSITIONS
        .iter()
        .map(|f| f.parse().expect("valid FEN"))
        .collect();

    for (i, board) in boards.iter().enumerate() {
        let legal = legal_set(board);

        for (j, other) in boards.iter().enumerate() {
            if i == j {
                continue;
            }
            for mv in legal_set(other) {
                assert_eq!(
                    movegen::is_legal(board, &mv),
                    legal.contains(&mv),
                    "is_legal disagreed on {mv} borrowed from {} when judged in {}",
                    POSITIONS[j],
                    POSITIONS[i]
                );
            }
        }
    }
}

/// The null move must never be playable — it encodes a1a1 and would otherwise
/// look like a quiet king move in some positions.
#[test]
fn test_null_move_is_never_legal() {
    for fen in POSITIONS {
        let board: Board = fen.parse().expect("valid FEN");
        assert!(
            !movegen::is_legal(&board, &Move::NULL),
            "Move::NULL accepted in {fen}"
        );
    }
}
