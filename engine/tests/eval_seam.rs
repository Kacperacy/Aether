//! The evaluation seam must admit an evaluator that is nothing like the
//! piece-square one.
//!
//! This is the property the NNUE work depends on: a network keeps a
//! hidden-layer accumulator, not middlegame/endgame sums, and it must be able to
//! plug in without `board` or `search` knowing. If this file stops compiling,
//! the seam has closed up again.

use aether_core::{Color, Move, Piece};
use board::Board;
use engine::eval::{Accumulator, Evaluator, Score, decode_move};
use engine::search::SearchLimits;
use engine::search::alpha_beta::AlphaBetaSearcher;

/// An accumulator with a different shape to `EvalState`: a single running
/// count, maintained purely through the piece add/remove primitive. It shares no
/// representation with the PST accumulator.
#[derive(Debug, Default)]
struct PieceCountAcc {
    /// White pieces minus black pieces, in units of one.
    balance: i32,
    stack: Vec<i32>,
}

impl Accumulator for PieceCountAcc {
    fn empty() -> Self {
        Self::default()
    }

    fn reset(&mut self, board: &Board) {
        self.balance = 0;
        self.stack.clear();

        for color in Color::ALL {
            for piece in [
                Piece::Pawn,
                Piece::Knight,
                Piece::Bishop,
                Piece::Rook,
                Piece::Queen,
            ] {
                let n = board.piece_count(piece, color) as i32;
                self.balance += if color == Color::White { n } else { -n };
            }
        }
    }

    fn push(&mut self, board: &Board, mv: &Move) {
        self.stack.push(self.balance);

        let Some(delta) = decode_move(board, mv) else {
            return;
        };

        // Exactly the primitive an NNUE accumulator would consume.
        for change in delta.iter() {
            if change.piece == Piece::King {
                continue;
            }
            let sign = if change.color == Color::White { 1 } else { -1 };
            self.balance += if change.added { sign } else { -sign };
        }
    }

    fn push_null(&mut self) {
        self.stack.push(self.balance);
    }

    fn pop(&mut self) {
        if let Some(v) = self.stack.pop() {
            self.balance = v;
        }
    }
}

struct PieceCountEvaluator;

impl Evaluator for PieceCountEvaluator {
    type Acc = PieceCountAcc;

    fn evaluate(&self, board: &Board, acc: &PieceCountAcc) -> Score {
        let from_white = acc.balance * 100;
        if board.side_to_move() == Color::White {
            from_white
        } else {
            -from_white
        }
    }
}

/// The seam admits a foreign accumulator type, and the search drives it.
#[test]
fn test_a_foreign_accumulator_can_drive_the_search() {
    let mut searcher = AlphaBetaSearcher::new(PieceCountEvaluator, 1);
    let mut board = Board::starting_position().unwrap();

    let result = searcher.search(&mut board, &SearchLimits::depth(4), |_, _, _| {});

    assert!(
        result.best_move.is_some(),
        "search produced no move under a custom evaluator"
    );
    assert!(result.info.nodes > 0);
}

/// Push/pop must be symmetric for the custom accumulator too — that discipline
/// belongs to the search, not to any one evaluator.
#[test]
fn test_custom_accumulator_is_restored_by_unmake() {
    let mut board: Board = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        .parse()
        .unwrap();

    let mut acc = PieceCountAcc::empty();
    acc.reset(&board);
    let before = acc.balance;

    let mut moves = movegen::MoveList::new();
    movegen::legal(&board, &mut moves);

    for mv in &moves {
        acc.push(&board, mv);
        board.make_move(mv).unwrap();
        board.unmake_move(mv).unwrap();
        acc.pop();

        assert_eq!(acc.balance, before, "accumulator drifted after {mv}");
    }
}

/// A capture must be seen by the accumulator through the decode alone.
#[test]
fn test_captures_reach_the_accumulator() {
    let mut board: Board = "7k/8/8/3q4/4P3/8/8/K7 w - - 0 1".parse().unwrap();

    let mut acc = PieceCountAcc::empty();
    acc.reset(&board);
    let before = acc.balance;

    let capture = Move::new(
        aether_core::Square::E4,
        aether_core::Square::D5,
        Move::CAPTURE,
    );
    acc.push(&board, &capture);
    board.make_move(&capture).unwrap();

    assert_eq!(
        acc.balance,
        before + 1,
        "capturing a black queen should move the balance by one"
    );
}
