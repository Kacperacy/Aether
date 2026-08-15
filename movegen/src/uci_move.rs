//! Parsing UCI long-algebraic moves against a position.
//!
//! A packed [`Move`] carries capture/en-passant/castle/promotion flags that a
//! bare "e2e4" string does not, so the only way to build a correct one is to
//! match the string against the position's legal moves. That makes this a
//! move-generation concern, not a protocol one.

use crate::legal;
use aether_core::{Move, Piece, Square};
use board::Board;
use std::str::FromStr;

/// Resolve a UCI move string (`"e2e4"`, `"e7e8q"`) to the matching legal move.
///
/// Returns `None` when the string is malformed or names no legal move.
#[must_use]
pub fn parse_uci_move(board: &Board, move_str: &str) -> Option<Move> {
    if move_str.len() < 4 {
        return None;
    }

    let from = Square::from_str(&move_str[0..2]).ok()?;
    let to = Square::from_str(&move_str[2..4]).ok()?;
    let promotion = match move_str.chars().nth(4) {
        Some(c) => Some(Piece::from_char(c)?),
        None => None,
    };

    let mut moves = Vec::new();
    legal(board, &mut moves);

    moves
        .into_iter()
        .find(|m| m.from_sq() == from && m.to_sq() == to && m.promotion_piece() == promotion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use board::STARTING_POSITION_FEN;

    #[test]
    fn test_parses_quiet_move() {
        let board: Board = STARTING_POSITION_FEN.parse().unwrap();
        let mv = parse_uci_move(&board, "e2e4").expect("e2e4 is legal");
        assert_eq!(mv.from_sq(), Square::E2);
        assert_eq!(mv.to_sq(), Square::E4);
        // Flags must come from generation, not the string.
        assert_eq!(mv.flags(), Move::DOUBLE_PUSH);
    }

    #[test]
    fn test_parses_promotion_with_correct_flag() {
        let board: Board = "8/P7/8/8/8/8/8/4K2k w - - 0 1".parse().unwrap();
        let mv = parse_uci_move(&board, "a7a8q").expect("a7a8q is legal");
        assert_eq!(mv.promotion_piece(), Some(Piece::Queen));
        assert!(mv.is_promotion());
    }

    #[test]
    fn test_rejects_illegal_and_malformed() {
        let board: Board = STARTING_POSITION_FEN.parse().unwrap();
        assert!(parse_uci_move(&board, "e2e5").is_none());
        assert!(parse_uci_move(&board, "zz").is_none());
        assert!(parse_uci_move(&board, "e2e9").is_none());
    }
}
