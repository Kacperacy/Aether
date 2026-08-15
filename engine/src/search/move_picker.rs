//! Lazy move selection.
//!
//! The search used to sort *every* move at *every* node with
//! `sort_by_cached_key`, which heap-allocates. Most nodes fail high on the first
//! or second move, so nearly all of that ordering work — and the allocation
//! behind it — was thrown away.
//!
//! [`MovePicker`] instead extracts the best remaining move on demand. Nodes that
//! cut off early stop paying for the tail of the list, and nothing allocates.
//!
//! # Why the order is bit-identical to the old sort
//!
//! Selecting the maximum and swapping it into place is *not* a stable sort: it
//! disturbs the order of the elements it swaps past, so equally-scored moves come
//! out in a different order than `sort_by_cached_key` produced. That matters more
//! than it sounds — ties are common (every quiet move with no history scores 0),
//! and generation order is what breaks them, so a different tie-break means a
//! different search tree and a different node count.
//!
//! So the sort key carries the move's original position as a low-order
//! tie-breaker, and travels with the move when it is swapped. Selecting the
//! maximum key then yields exactly stable-descending-by-score order, and the node
//! count is unchanged — which is what makes this a pure speed change with an
//! exact regression gate rather than something needing a match to validate.

use crate::search::move_ordering::MoveOrderer;
use aether_core::Move;
use board::Board;
use movegen::{MAX_MOVES, MoveList};

/// Bits reserved for the tie-breaker. `MAX_MOVES` is 256, so 16 is generous and
/// keeps the arithmetic obvious.
const TIEBREAK_BITS: u32 = 16;

const _: () = assert!(
    MAX_MOVES < (1 << TIEBREAK_BITS),
    "tie-breaker must not overflow into the score bits"
);

pub(crate) struct MovePicker {
    moves: MoveList,
    /// `(score << TIEBREAK_BITS) + (MAX_MOVES - original_index)`.
    keys: [i64; MAX_MOVES],
    index: usize,
}

impl MovePicker {
    fn score_all(
        moves: MoveList,
        orderer: &MoveOrderer,
        tt_move: Option<Move>,
        ply: usize,
        board: &Board,
    ) -> Self {
        let mut keys = [0i64; MAX_MOVES];

        for (i, mv) in moves.iter().enumerate() {
            let score = i64::from(orderer.move_score_with_see(mv, tt_move, ply, board));
            // Earlier moves get the larger tie-breaker, so equal scores come out
            // in generation order — the same order a stable sort would give.
            keys[i] = (score << TIEBREAK_BITS) + (MAX_MOVES - i) as i64;
        }

        Self {
            moves,
            keys,
            index: 0,
        }
    }

    /// All legal moves at an interior node.
    pub fn new(board: &Board, orderer: &MoveOrderer, tt_move: Option<Move>, ply: usize) -> Self {
        let mut moves = MoveList::new();
        movegen::legal(board, &mut moves);
        Self::score_all(moves, orderer, tt_move, ply, board)
    }

    /// The quiescence move set: evasions when in check, otherwise captures —
    /// plus quiet checks at the horizon.
    pub fn quiescence(
        board: &Board,
        orderer: &MoveOrderer,
        ply: usize,
        in_check: bool,
        include_checks: bool,
    ) -> Self {
        let mut moves = MoveList::new();
        if in_check {
            movegen::legal(board, &mut moves);
        } else {
            movegen::captures(board, &mut moves);
            if include_checks {
                movegen::checks(board, &mut moves);
            }
        }
        Self::score_all(moves, orderer, None, ply, board)
    }

    /// True when the position produced no moves at all — checkmate or stalemate
    /// at an interior node.
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// The best remaining move, or `None` when exhausted.
    pub fn next(&mut self) -> Option<Move> {
        let len = self.moves.len();
        if self.index >= len {
            return None;
        }

        let mut best = self.index;
        for i in (self.index + 1)..len {
            if self.keys[i] > self.keys[best] {
                best = i;
            }
        }

        self.moves.swap(self.index, best);
        self.keys.swap(self.index, best);

        let mv = self.moves[self.index];
        self.index += 1;
        Some(mv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::test_support::pos;
    use std::collections::HashSet;

    const POSITIONS: &[&str] = &[
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1",
        // In check, with evasions available.
        "4k3/8/8/8/7b/8/8/4K3 w - - 0 1",
        // Fool's mate — in check with *no* legal moves at all.
        "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3",
    ];

    fn drain(picker: &mut MovePicker) -> Vec<Move> {
        let mut out = Vec::new();
        while let Some(mv) = picker.next() {
            out.push(mv);
        }
        out
    }

    /// The picker must yield every legal move exactly once — no duplicates, and
    /// nothing dropped. Everything else it does is worthless if this fails.
    #[test]
    fn test_yields_every_legal_move_exactly_once() {
        let orderer = MoveOrderer::new();

        for fen in POSITIONS {
            let board = pos(fen);
            let mut expected = MoveList::new();
            movegen::legal(&board, &mut expected);

            let yielded = drain(&mut MovePicker::new(&board, &orderer, None, 0));

            assert_eq!(yielded.len(), expected.len(), "wrong move count in {fen}");
            assert_eq!(
                yielded.iter().copied().collect::<HashSet<_>>(),
                expected.iter().copied().collect::<HashSet<_>>(),
                "picker did not yield the legal move set in {fen}"
            );
        }
    }

    /// The ordering contract: identical to a stable descending sort by score.
    /// This is what keeps the node count bit-identical.
    #[test]
    fn test_order_matches_a_stable_descending_sort() {
        let orderer = MoveOrderer::new();

        for fen in POSITIONS {
            let board = pos(fen);

            let mut expected = MoveList::new();
            movegen::legal(&board, &mut expected);
            orderer.order_moves_with_see(&mut expected, None, 0, &board);

            let yielded = drain(&mut MovePicker::new(&board, &orderer, None, 0));

            assert_eq!(
                yielded,
                expected.iter().copied().collect::<Vec<_>>(),
                "picker order diverged from the sorted order in {fen}"
            );
        }
    }

    /// Same contract, with a TT move forced to the front.
    #[test]
    fn test_tt_move_is_yielded_first() {
        let orderer = MoveOrderer::new();

        for fen in POSITIONS {
            let board = pos(fen);
            let mut legal = MoveList::new();
            movegen::legal(&board, &mut legal);

            // Checkmate positions have nothing to promote to the front.
            let Some(&tt_move) = legal.last() else {
                continue;
            };

            let mut picker = MovePicker::new(&board, &orderer, Some(tt_move), 0);
            assert_eq!(
                picker.next(),
                Some(tt_move),
                "TT move was not yielded first in {fen}"
            );
        }
    }

    #[test]
    fn test_quiescence_picker_yields_only_captures_when_not_in_check() {
        let orderer = MoveOrderer::new();
        let board = pos("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");

        let yielded = drain(&mut MovePicker::quiescence(
            &board, &orderer, 0, false, false,
        ));

        assert!(!yielded.is_empty());
        assert!(
            yielded.iter().all(|m| m.is_capture() || m.is_en_passant()),
            "quiescence picker yielded a quiet move"
        );
    }

    #[test]
    fn test_empty_position_yields_nothing() {
        let orderer = MoveOrderer::new();
        // Stalemate: black to move, no legal moves.
        let board = pos("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");

        let mut picker = MovePicker::new(&board, &orderer, None, 0);
        assert!(picker.is_empty());
        assert_eq!(picker.next(), None);
    }
}
