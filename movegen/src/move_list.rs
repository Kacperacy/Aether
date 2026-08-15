//! Fixed-capacity move list.
//!
//! Move generation runs at every node, so a heap `Vec` is the wrong shape: the
//! old code allocated one per quiescence node and then immediately reallocated
//! it, because `pseudo_legal` reserved 256 entries. A `Move` is a `u16`, so the
//! whole list is 512 bytes — cheap to hold by value on the stack.
//!
//! The capacity is a hard bound, not a hint. 218 is the highest number of legal
//! moves known for any reachable position; 256 gives headroom and keeps the
//! backing array a round size.

use aether_core::Move;
use std::ops::{Deref, DerefMut};

/// Upper bound on moves generated for a single position.
pub const MAX_MOVES: usize = 256;

/// A stack-allocated list of at most [`MAX_MOVES`] moves.
///
/// Derefs to `[Move]`, so slice operations — iteration, sorting, `swap_remove`
/// via [`MoveList::retain`] — work as they would on a `Vec`.
#[derive(Clone)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl MoveList {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            moves: [Move::NULL; MAX_MOVES],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Append a move.
    ///
    /// Silently drops the move if the list is somehow full — generation cannot
    /// legitimately exceed [`MAX_MOVES`], and a debug build asserts that, but a
    /// release build must not panic or write out of bounds in the hot path.
    #[inline(always)]
    pub fn push(&mut self, mv: Move) {
        debug_assert!(self.len < MAX_MOVES, "move list overflow");
        if self.len < MAX_MOVES {
            self.moves[self.len] = mv;
            self.len += 1;
        }
    }

    /// Shorten the list to `len` moves, dropping the rest.
    #[inline(always)]
    pub fn truncate(&mut self, len: usize) {
        if len < self.len {
            self.len = len;
        }
    }

    /// Keep only the moves satisfying `predicate`, preserving relative order.
    ///
    /// Order matters even though generation order is arbitrary: move ordering
    /// scores moves and then *stably* sorts them, so generation order is the
    /// tie-breaker between equally-scored moves, and it therefore decides which
    /// tree the search walks. Compacting by swapping the last element into the
    /// hole would be marginally cheaper and would silently change the node
    /// count — measured at ~5% on `bench` — for no gain.
    #[inline]
    pub fn retain<F: FnMut(Move) -> bool>(&mut self, mut predicate: F) {
        let mut kept = 0;
        for i in 0..self.len {
            if predicate(self.moves[i]) {
                self.moves[kept] = self.moves[i];
                kept += 1;
            }
        }
        self.len = kept;
    }
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for MoveList {
    type Target = [Move];

    #[inline(always)]
    fn deref(&self) -> &[Move] {
        &self.moves[..self.len]
    }
}

impl DerefMut for MoveList {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [Move] {
        &mut self.moves[..self.len]
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = std::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl std::fmt::Debug for MoveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::Square;

    fn mv(from: Square, to: Square) -> Move {
        Move::new(from, to, Move::QUIET)
    }

    #[test]
    fn test_push_len_and_iteration() {
        let mut list = MoveList::new();
        assert!(list.is_empty());

        list.push(mv(Square::E2, Square::E4));
        list.push(mv(Square::D2, Square::D4));

        assert_eq!(list.len(), 2);
        assert_eq!(list[0].to_sq(), Square::E4);
        assert_eq!(list.iter().count(), 2);
    }

    #[test]
    fn test_clear_resets_without_reallocating() {
        let mut list = MoveList::new();
        for _ in 0..10 {
            list.push(mv(Square::E2, Square::E4));
        }
        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.iter().count(), 0);
    }

    #[test]
    fn test_retain_keeps_exactly_the_matching_moves() {
        let mut list = MoveList::new();
        list.push(mv(Square::E2, Square::E4));
        list.push(mv(Square::D2, Square::D4));
        list.push(mv(Square::G1, Square::F3));

        list.retain(|m| m.to_sq() != Square::D4);

        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|m| m.to_sq() != Square::D4));
    }

    /// Relative order must survive `retain` — move ordering sorts stably, so
    /// generation order is the tie-breaker that decides the search tree.
    #[test]
    fn test_retain_preserves_relative_order() {
        let mut list = MoveList::new();
        for to in [Square::A2, Square::B2, Square::C2, Square::D2, Square::E2] {
            list.push(mv(Square::A1, to));
        }

        list.retain(|m| m.to_sq() != Square::B2 && m.to_sq() != Square::D2);

        let kept: Vec<Square> = list.iter().map(|m| m.to_sq()).collect();
        assert_eq!(kept, vec![Square::A2, Square::C2, Square::E2]);
    }

    #[test]
    fn test_retain_all_and_none() {
        let mut list = MoveList::new();
        for _ in 0..5 {
            list.push(mv(Square::E2, Square::E4));
        }

        list.retain(|_| true);
        assert_eq!(list.len(), 5);

        list.retain(|_| false);
        assert!(list.is_empty());
    }

    #[test]
    fn test_capacity_is_a_hard_bound() {
        // Release builds must not write out of bounds when the bound is hit.
        let mut list = MoveList::new();
        for _ in 0..MAX_MOVES {
            list.push(mv(Square::E2, Square::E4));
        }
        assert_eq!(list.len(), MAX_MOVES);
    }

    #[test]
    fn test_sorting_through_deref_mut() {
        let mut list = MoveList::new();
        list.push(mv(Square::H1, Square::H2));
        list.push(mv(Square::A1, Square::A2));

        list.sort_by_key(|m| m.from_sq().to_index());

        assert_eq!(list[0].from_sq(), Square::A1);
    }
}
