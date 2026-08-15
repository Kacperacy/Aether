mod builder;
mod cache;
mod error;
mod fen;
mod ops;
mod query;
mod state_info;
mod zobrist;
mod zobrist_keys;

pub use builder::BoardBuilder;
pub use error::{BoardError, FenError};
pub use fen::STARTING_POSITION_FEN;

use aether_core::{BitBoard, Color, File, Piece, Rank, Square};
use cache::BoardCache;
use state_info::StateInfo;

pub type Result<T> = std::result::Result<T, BoardError>;

/// Capacity of the unmake stack, in plies.
///
/// `state_history` is indexed `history_index % MAX_SEARCH_DEPTH`, so the
/// invariant is that no more than this many moves are *live* (made but not yet
/// unmade) at once. Search is bounded well under this, but a replayed game is
/// not — moves committed by `position ... moves ...` are never unmade, so
/// [`Board::commit_history`] drops them from the stack rather than letting them
/// consume (and eventually wrap) it.
const MAX_SEARCH_DEPTH: usize = 256;
const ZOBRIST_HISTORY_CAPACITY: usize = 512;
const FIFTY_MOVE_THRESHOLD: u16 = 100;

#[derive(Debug, Clone)]
pub struct Board {
    pieces: [[BitBoard; Piece::NUM]; Color::NUM],
    mailbox: [Option<(Piece, Color)>; Square::NUM],
    cache: BoardCache,

    side_to_move: Color,
    fullmove_number: u16,
    state: StateInfo,

    state_history: [StateInfo; MAX_SEARCH_DEPTH],
    history_index: usize,

    zobrist_history: Vec<u64>,
}

impl Board {
    pub fn starting_position() -> Result<Self> {
        BoardBuilder::starting_position().build()
    }

    /// Mark every move played so far as permanent, emptying the unmake stack.
    ///
    /// Call this once a position is fully set up (e.g. after replaying a game's
    /// moves) — those moves will never be unmade, so keeping them on the stack
    /// only eats into the depth available to search. Repetition detection is
    /// unaffected: it reads `zobrist_history`, which this leaves intact.
    #[inline]
    pub fn commit_history(&mut self) {
        self.history_index = 0;
    }

    pub fn as_ascii(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        writeln!(out).unwrap();
        for rank in (0..Rank::NUM as i8).rev() {
            write!(out, "{}", rank + 1).unwrap();
            for file in 0..File::NUM as i8 {
                let sq = Square::new(File::from_index(file), Rank::from_index(rank));
                let ch = self.piece_at(sq).map_or('.', |(p, c)| {
                    let ch = p.as_char();
                    if c == Color::White {
                        ch.to_ascii_uppercase()
                    } else {
                        ch
                    }
                });
                write!(out, " {ch}").unwrap();
            }
            writeln!(out).unwrap();
        }
        writeln!(out, "  A B C D E F G H").unwrap();
        out
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::starting_position().expect("Failed to create starting position")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aether_core::{Move, Square};

    #[test]
    fn test_repetition_count_no_repetition() {
        let mut board = Board::starting_position().unwrap();

        // Make moves that don't repeat: 1. Nf3 Nf6 2. Nc3 Nc6
        let moves = [
            Move::new(Square::G1, Square::F3, Move::QUIET),
            Move::new(Square::G8, Square::F6, Move::QUIET),
            Move::new(Square::B1, Square::C3, Move::QUIET),
            Move::new(Square::B8, Square::C6, Move::QUIET),
        ];

        for mv in &moves {
            board.make_move(mv).unwrap();
        }

        assert_eq!(board.repetition_count(), 0);
        assert!(!board.is_twofold_repetition());
        assert!(!board.is_threefold_repetition());
    }

    #[test]
    fn test_twofold_repetition_detected() {
        let mut board = Board::starting_position().unwrap();

        // Create twofold repetition with knight shuffle:
        // 1. Nf3 Nf6 2. Ng1 Ng8 - back to starting position (2nd occurrence)
        let moves = [
            Move::new(Square::G1, Square::F3, Move::QUIET),
            Move::new(Square::G8, Square::F6, Move::QUIET),
            Move::new(Square::F3, Square::G1, Move::QUIET),
            Move::new(Square::F6, Square::G8, Move::QUIET),
        ];

        for mv in &moves {
            board.make_move(mv).unwrap();
        }

        // Now we're back at starting position - should be twofold
        assert_eq!(board.repetition_count(), 1);
        assert!(board.is_twofold_repetition());
        assert!(!board.is_threefold_repetition());
    }

    #[test]
    fn test_threefold_repetition_detected() {
        let mut board = Board::starting_position().unwrap();

        // Create threefold repetition with knight shuffle:
        // 1. Nf3 Nf6 2. Ng1 Ng8 (2nd) 3. Nf3 Nf6 4. Ng1 Ng8 (3rd)
        let moves = [
            // First cycle
            Move::new(Square::G1, Square::F3, Move::QUIET),
            Move::new(Square::G8, Square::F6, Move::QUIET),
            Move::new(Square::F3, Square::G1, Move::QUIET),
            Move::new(Square::F6, Square::G8, Move::QUIET),
            // Second cycle
            Move::new(Square::G1, Square::F3, Move::QUIET),
            Move::new(Square::G8, Square::F6, Move::QUIET),
            Move::new(Square::F3, Square::G1, Move::QUIET),
            Move::new(Square::F6, Square::G8, Move::QUIET),
        ];

        for mv in &moves {
            board.make_move(mv).unwrap();
        }

        // Now we're at starting position for 3rd time
        assert_eq!(board.repetition_count(), 2);
        assert!(board.is_twofold_repetition());
        assert!(board.is_threefold_repetition());
    }

    #[test]
    fn test_repetition_broken_by_pawn_move() {
        let mut board = Board::starting_position().unwrap();

        // 1. Nf3 Nf6 2. Ng1 Ng8 (2nd occurrence of start)
        let moves_cycle1 = [
            Move::new(Square::G1, Square::F3, Move::QUIET),
            Move::new(Square::G8, Square::F6, Move::QUIET),
            Move::new(Square::F3, Square::G1, Move::QUIET),
            Move::new(Square::F6, Square::G8, Move::QUIET),
        ];

        for mv in &moves_cycle1 {
            board.make_move(mv).unwrap();
        }

        assert!(board.is_twofold_repetition());

        // Now make a pawn move which resets halfmove clock
        let pawn_move = Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH);
        board.make_move(&pawn_move).unwrap();

        // After pawn move, repetition count should be 0
        assert_eq!(board.repetition_count(), 0);
        assert!(!board.is_twofold_repetition());
    }

    #[test]
    fn test_repetition_only_counts_same_side_to_move() {
        let mut board = Board::starting_position().unwrap();

        // 1. Nf3 - white moved, position changed
        let mv1 = Move::new(Square::G1, Square::F3, Move::QUIET);
        board.make_move(&mv1).unwrap();

        // 1... Nf6 - black moved
        let mv2 = Move::new(Square::G8, Square::F6, Move::QUIET);
        board.make_move(&mv2).unwrap();

        // 2. Ng1 - white moved back
        let mv3 = Move::new(Square::F3, Square::G1, Move::QUIET);
        board.make_move(&mv3).unwrap();

        // Position after 1. Nf3 and after 2. Ng1 are different
        // (black knight on f6 vs g8), so no repetition yet
        // But more importantly, we're checking that we only compare
        // positions with the same side to move

        // 2... Ng8 - back to starting position with white to move
        let mv4 = Move::new(Square::F6, Square::G8, Move::QUIET);
        board.make_move(&mv4).unwrap();

        // Now it's white to move, same as starting position
        assert_eq!(board.repetition_count(), 1);
        assert!(board.is_twofold_repetition());
    }

    #[test]
    fn test_fifty_move_draw_detected() {
        let mut board = Board::starting_position().unwrap();

        // Set halfmove clock to 100
        board.state.halfmove_clock = 100;

        assert!(board.is_fifty_move_draw());
    }

    #[test]
    fn test_insufficient_material_king_vs_king() {
        let board: Board = "8/8/8/4k3/8/8/8/4K3 w - - 0 1".parse().unwrap();
        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_kb_vs_k() {
        let board: Board = "8/8/8/4k3/8/8/2B5/4K3 w - - 0 1".parse().unwrap();
        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_kn_vs_k() {
        let board: Board = "8/8/8/4k3/8/8/2N5/4K3 w - - 0 1".parse().unwrap();
        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_kq_vs_k() {
        let board: Board = "8/8/8/4k3/8/8/2Q5/4K3 w - - 0 1".parse().unwrap();
        assert!(!board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_kr_vs_k() {
        let board: Board = "8/8/8/4k3/8/8/2R5/4K3 w - - 0 1".parse().unwrap();
        assert!(!board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_kbn_vs_k() {
        let board: Board = "8/8/8/4k3/8/8/2BN4/4K3 w - - 0 1".parse().unwrap();
        assert!(!board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_kb_vs_kb_same_color() {
        let board: Board = "8/8/8/4k3/8/2b5/8/B3K3 w - - 0 1".parse().unwrap();
        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_kb_vs_kb_different_color() {
        let board: Board = "8/8/8/4k3/8/2b5/8/1B2K3 w - - 0 1".parse().unwrap();
        assert!(!board.is_insufficient_material());
    }

    #[test]
    fn test_ply_tracking() {
        let mut board = Board::starting_position().unwrap();
        assert_eq!(board.ply(), 0);

        let moves = [
            Move::new(Square::E2, Square::E4, Move::DOUBLE_PUSH),
            Move::new(Square::E7, Square::E5, Move::DOUBLE_PUSH),
            Move::new(Square::G1, Square::F3, Move::QUIET),
        ];

        for (i, mv) in moves.iter().enumerate() {
            board.make_move(mv).unwrap();
            assert_eq!(board.ply(), i + 1);
        }

        // Unmake and verify ply decreases
        for (i, mv) in moves.iter().rev().enumerate() {
            board.unmake_move(mv).unwrap();
            assert_eq!(board.ply(), moves.len() - i - 1);
        }

        assert_eq!(board.ply(), 0);
    }

    #[test]
    fn test_circular_buffer_make_unmake_deep() {
        let mut board = Board::starting_position().unwrap();

        // Make many moves to test circular buffer doesn't overflow
        let move_sequence = [
            Move::new(Square::G1, Square::F3, Move::QUIET),
            Move::new(Square::G8, Square::F6, Move::QUIET),
            Move::new(Square::F3, Square::G1, Move::QUIET),
            Move::new(Square::F6, Square::G8, Move::QUIET),
        ];

        // 200 plies of make/unmake churn through the ring buffer.
        for _ in 0..50 {
            for mv in &move_sequence {
                board.make_move(mv).unwrap();
            }
        }

        assert_eq!(board.ply(), 200);

        // Unmake 100 moves - should work correctly with circular buffer
        for _ in 0..25 {
            for mv in move_sequence.iter().rev() {
                board.unmake_move(mv).unwrap();
            }
        }

        assert_eq!(board.ply(), 100);
    }
}
