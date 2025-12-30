mod builder;
mod cache;
mod error;
mod fen;
mod game_state;
mod ops;
mod query;
mod zobrist;

pub use builder::BoardBuilder;
pub use error::{BoardError, FenError, MoveError};
pub use fen::{FenOps, STARTING_POSITION_FEN};
pub use ops::BoardOps;
pub use query::BoardQuery;

use aether_core::{BitBoard, Color, File, MoveState, Piece, Rank, Square, attackers_to_square};
use cache::BoardCache;
use game_state::GameState;
use std::num::NonZeroU64;

pub type Result<T> = std::result::Result<T, BoardError>;

const MAX_SEARCH_DEPTH: usize = 256;

#[derive(Debug, Clone)]
pub struct Board {
    pieces: [[BitBoard; 6]; 2],
    game_state: GameState,
    cache: BoardCache,
    zobrist_hash: u64,
    /// Stack to store move states for unmake operations
    move_history: [MoveState; MAX_SEARCH_DEPTH],
    history_count: usize,
    /// Mailbox representation for easy piece lookup
    mailbox: [Option<(Piece, Color)>; 64],
}

impl Board {
    pub fn empty() -> Self {
        Self {
            pieces: [[BitBoard::EMPTY; 6]; 2],
            game_state: GameState::new(),
            cache: BoardCache::new(),
            zobrist_hash: 0,
            move_history: [MoveState::default(); MAX_SEARCH_DEPTH],
            history_count: 0,
            mailbox: [None; 64],
        }
    }

    pub fn starting_position() -> Result<Self> {
        BoardBuilder::starting_position().build()
    }

    pub fn builder() -> BoardBuilder {
        BoardBuilder::new()
    }

    #[inline]
    pub fn pieces(&self) -> &[[BitBoard; 6]; 2] {
        &self.pieces
    }

    #[inline]
    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    #[inline]
    pub fn zobrist_hash(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.zobrist_hash)
    }

    #[inline]
    pub fn zobrist_hash_raw(&self) -> u64 {
        self.zobrist_hash
    }

    #[inline]
    pub fn move_history(&self) -> &[MoveState] {
        &self.move_history
    }

    pub fn piece_at_fast(&self, square: Square) -> Option<(Piece, Color)> {
        self.mailbox[square.to_index() as usize]
    }

    #[inline]
    pub fn cache(&self) -> &BoardCache {
        &self.cache
    }

    #[inline]
    pub fn refresh_cache(&mut self) {
        self.cache.refresh(&self.pieces);
    }

    #[inline]
    pub fn attackers_to_square(&self, sq: Square, color: Color) -> BitBoard {
        attackers_to_square(sq, color, self.cache.occupied, &self.pieces[color as usize])
    }

    #[inline]
    pub fn occupied(&self) -> BitBoard {
        self.cache.occupied
    }

    #[inline]
    pub fn color_occupied(&self, color: Color) -> BitBoard {
        self.cache.color_combined[color as usize]
    }

    pub fn print(&self) {
        println!("{}", self.as_ascii());
    }

    pub fn as_ascii(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        writeln!(out).unwrap();
        for rank in (0..8).rev() {
            write!(out, "{}", rank + 1).unwrap();
            for file in 0..8 {
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

    #[inline]
    pub fn ply(&self) -> usize {
        self.move_history.len()
    }

    pub fn repetition_count(&self) -> usize {
        let current_hash = self.zobrist_hash;
        let mut count = 0;

        let start_idx = self
            .history_count
            .saturating_sub(self.game_state.halfmove_clock as usize);

        for i in (start_idx..self.history_count).step_by(2) {
            if self.move_history[i].old_zobrist_hash == current_hash {
                count += 1;
            }
        }

        count
    }

    #[inline]
    pub fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        <Self as BoardQuery>::piece_at(self, square)
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
    use crate::FenOps;
    use aether_core::{Move, Piece, Square};

    #[test]
    fn test_repetition_count_no_repetition() {
        let mut board = Board::starting_position().unwrap();

        // Make moves that don't repeat: 1. Nf3 Nf6 2. Nc3 Nc6
        let moves = [
            Move::new(Square::G1, Square::F3, Piece::Knight),
            Move::new(Square::G8, Square::F6, Piece::Knight),
            Move::new(Square::B1, Square::C3, Piece::Knight),
            Move::new(Square::B8, Square::C6, Piece::Knight),
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
            Move::new(Square::G1, Square::F3, Piece::Knight),
            Move::new(Square::G8, Square::F6, Piece::Knight),
            Move::new(Square::F3, Square::G1, Piece::Knight),
            Move::new(Square::F6, Square::G8, Piece::Knight),
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
            Move::new(Square::G1, Square::F3, Piece::Knight),
            Move::new(Square::G8, Square::F6, Piece::Knight),
            Move::new(Square::F3, Square::G1, Piece::Knight),
            Move::new(Square::F6, Square::G8, Piece::Knight),
            // Second cycle
            Move::new(Square::G1, Square::F3, Piece::Knight),
            Move::new(Square::G8, Square::F6, Piece::Knight),
            Move::new(Square::F3, Square::G1, Piece::Knight),
            Move::new(Square::F6, Square::G8, Piece::Knight),
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
            Move::new(Square::G1, Square::F3, Piece::Knight),
            Move::new(Square::G8, Square::F6, Piece::Knight),
            Move::new(Square::F3, Square::G1, Piece::Knight),
            Move::new(Square::F6, Square::G8, Piece::Knight),
        ];

        for mv in &moves_cycle1 {
            board.make_move(mv).unwrap();
        }

        assert!(board.is_twofold_repetition());

        // Now make a pawn move which resets halfmove clock
        let pawn_move =
            Move::new(Square::E2, Square::E4, Piece::Pawn).with_flags(aether_core::MoveFlags {
                is_double_pawn_push: true,
                is_castle: false,
                is_en_passant: false,
            });
        board.make_move(&pawn_move).unwrap();

        // After pawn move, repetition count should be 0
        assert_eq!(board.repetition_count(), 0);
        assert!(!board.is_twofold_repetition());
    }

    #[test]
    fn test_repetition_only_counts_same_side_to_move() {
        let mut board = Board::starting_position().unwrap();

        // 1. Nf3 - white moved, position changed
        let mv1 = Move::new(Square::G1, Square::F3, Piece::Knight);
        board.make_move(&mv1).unwrap();

        // 1... Nf6 - black moved
        let mv2 = Move::new(Square::G8, Square::F6, Piece::Knight);
        board.make_move(&mv2).unwrap();

        // 2. Ng1 - white moved back
        let mv3 = Move::new(Square::F3, Square::G1, Piece::Knight);
        board.make_move(&mv3).unwrap();

        // Position after 1. Nf3 and after 2. Ng1 are different
        // (black knight on f6 vs g8), so no repetition yet
        // But more importantly, we're checking that we only compare
        // positions with the same side to move

        // 2... Ng8 - back to starting position with white to move
        let mv4 = Move::new(Square::F6, Square::G8, Piece::Knight);
        board.make_move(&mv4).unwrap();

        // Now it's white to move, same as starting position
        assert_eq!(board.repetition_count(), 1);
        assert!(board.is_twofold_repetition());
    }

    #[test]
    fn test_fifty_move_draw_detected() {
        let mut board = Board::starting_position().unwrap();

        // Set halfmove clock to 100
        board.game_state.halfmove_clock = 100;

        assert!(board.is_fifty_move_draw());
    }

    #[test]
    fn test_insufficient_material_king_vs_king() {
        // K vs K
        let fen = "8/8/8/4k3/8/8/8/4K3 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_kb_vs_k() {
        // K+B vs K
        let fen = "8/8/8/4k3/8/8/2B5/4K3 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_kn_vs_k() {
        // K+N vs K
        let fen = "8/8/8/4k3/8/8/2N5/4K3 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_kq_vs_k() {
        // K+Q vs K - sufficient (can mate)
        let fen = "8/8/8/4k3/8/8/2Q5/4K3 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();

        assert!(!board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_kr_vs_k() {
        // K+R vs K - sufficient (can mate)
        let fen = "8/8/8/4k3/8/8/2R5/4K3 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();

        assert!(!board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_kbn_vs_k() {
        // K+B+N vs K - sufficient (can mate)
        let fen = "8/8/8/4k3/8/8/2BN4/4K3 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();

        assert!(!board.is_insufficient_material());
    }

    #[test]
    fn test_insufficient_material_kb_vs_kb_same_color() {
        // K+B vs K+B on same color squares (both light squares)
        let fen = "8/8/3b4/4k3/8/8/2B5/4K3 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();

        // Both bishops on light squares (c2 and d6)
        // c2: (2+1) % 2 = 1 (dark)
        // d6: (3+5) % 2 = 0 (light)
        // Actually these are different colors, so NOT insufficient

        // Let's use correct squares: a1 (light) and c3 (light)
        let fen = "8/8/8/4k3/8/2b5/8/B3K3 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();
        // a1: (0+0) % 2 = 0 (light)
        // c3: (2+2) % 2 = 0 (light)

        assert!(board.is_insufficient_material());
    }

    #[test]
    fn test_sufficient_material_kb_vs_kb_different_color() {
        // K+B vs K+B on different color squares
        let fen = "8/8/8/4k3/8/2b5/8/1B2K3 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();
        // b1: (1+0) % 2 = 1 (dark)
        // c3: (2+2) % 2 = 0 (light)
        // Different colors - not insufficient

        assert!(!board.is_insufficient_material());
    }
}
