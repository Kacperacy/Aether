//! Every public generator must emit only legal moves.
//!
//! `legal()` filtered pins; `captures()` and `checks()` did not, and the search
//! consumed them directly in quiescence — where `Board::make_move` happily
//! applies an illegal move, because it never validates legality. The result was
//! that quiescence could move a pinned piece, and the reply could then capture
//! the king, corrupting the score by ~`KING_VALUE`.
//!
//! These tests hold every generator to the same contract.

use aether_core::Move;
use board::Board;

fn legal_set(board: &Board) -> movegen::MoveList {
    let mut moves = movegen::MoveList::new();
    movegen::legal(board, &mut moves);
    moves
}

/// Positions chosen so that a naive generator produces moves that `legal()`
/// rejects: absolute pins, en-passant pins along a rank, and checks.
const PINNED_POSITIONS: &[&str] = &[
    // Bishop on g5 pins the f6 knight to the king on e8.
    "rnbqkb1r/pppp1ppp/5n2/4p1B1/4P3/8/PPPP1PPP/RN1QKBNR b KQkq - 0 3",
    // Rook on e1 pins the e-file; black queen on e7 faces it through pieces.
    "4k3/4q3/8/8/8/8/8/4RK2 b - - 0 1",
    // Rook pins a knight against the king along a rank.
    "8/8/8/8/8/8/8/R2nk2K b - - 0 1",
    // En-passant capture that would expose the king along the 5th rank.
    "8/8/8/K2pP2r/8/8/8/7k w - d6 0 1",
    // Side to move is in check — only evasions are legal.
    "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
    // Bishop pin on the long diagonal.
    "rnbqk1nr/pppp1ppp/8/2b1p3/4P3/2N5/PPPP1PPP/R1BQKBNR w KQkq - 4 4",
];

#[test]
fn test_captures_are_legal() {
    for fen in PINNED_POSITIONS {
        let board: Board = fen.parse().expect("valid FEN");
        let legal = legal_set(&board);

        let mut captures = movegen::MoveList::new();
        movegen::captures(&board, &mut captures);

        for mv in &captures {
            assert!(
                legal.contains(mv),
                "captures() produced illegal move {mv} in {fen}"
            );
        }
    }
}

#[test]
fn test_checks_are_legal() {
    for fen in PINNED_POSITIONS {
        let board: Board = fen.parse().expect("valid FEN");
        let legal = legal_set(&board);

        let mut checks = movegen::MoveList::new();
        movegen::checks(&board, &mut checks);

        for mv in &checks {
            assert!(
                legal.contains(mv),
                "checks() produced illegal move {mv} in {fen}"
            );
        }
    }
}

/// A generator must never offer a move that captures a king — that position is
/// unreachable in legal play, and scoring it hands out `KING_VALUE`.
#[test]
fn test_no_generator_can_capture_a_king() {
    let mut all = PINNED_POSITIONS.to_vec();
    all.push("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    all.push("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

    for fen in all {
        let board: Board = fen.parse().expect("valid FEN");
        let enemy_king = board.get_king_square(board.side_to_move().opponent());

        for (name, moves) in [
            ("legal", {
                let mut m = movegen::MoveList::new();
                movegen::legal(&board, &mut m);
                m
            }),
            ("captures", {
                let mut m = movegen::MoveList::new();
                movegen::captures(&board, &mut m);
                m
            }),
            ("checks", {
                let mut m = movegen::MoveList::new();
                movegen::checks(&board, &mut m);
                m
            }),
        ] {
            for mv in &moves {
                assert_ne!(
                    mv.to_sq(),
                    enemy_king,
                    "{name}() produced a king capture {mv} in {fen}"
                );
            }
        }
    }
}

/// `captures()` must agree with filtering `legal()` — same set, no more, no less.
#[test]
fn test_captures_match_filtered_legal() {
    for fen in PINNED_POSITIONS {
        let board: Board = fen.parse().expect("valid FEN");

        let mut expected: Vec<Move> = legal_set(&board)
            .iter()
            .copied()
            .filter(|m| m.is_capture() || m.is_en_passant())
            .collect();

        let mut list = movegen::MoveList::new();
        movegen::captures(&board, &mut list);
        let mut actual: Vec<Move> = list.iter().copied().collect();

        expected.sort_by_key(|m| m.0);
        actual.sort_by_key(|m| m.0);

        assert_eq!(
            actual, expected,
            "captures() disagrees with legal() in {fen}"
        );
    }
}

/// `captures` and `quiets` must partition `legal` — that is what lets a staged
/// consumer take both and see every legal move exactly once.
#[test]
fn test_captures_and_quiets_partition_legal() {
    let mut positions = PINNED_POSITIONS.to_vec();
    positions.push("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    positions.push("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    positions.push("n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1");

    for fen in positions {
        let board: Board = fen.parse().expect("valid FEN");

        let legal: std::collections::HashSet<Move> = legal_set(&board).iter().copied().collect();

        let mut caps = movegen::MoveList::new();
        movegen::captures(&board, &mut caps);
        let mut quiet = movegen::MoveList::new();
        movegen::quiets(&board, &mut quiet);

        assert_eq!(
            caps.len() + quiet.len(),
            legal.len(),
            "captures + quiets must cover legal exactly once in {fen}"
        );

        let union: std::collections::HashSet<Move> =
            caps.iter().chain(quiet.iter()).copied().collect();
        assert_eq!(union, legal, "captures U quiets != legal in {fen}");
    }
}
