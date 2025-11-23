mod builder;
mod cache;
mod error;
mod fen;
mod game_state;
mod ops;
mod query;
mod zobrist;

pub use builder::BoardBuilder;
pub use fen::{FenOps, STARTING_POSITION_FEN};
pub use ops::BoardOps;
pub use query::BoardQuery;

use crate::error::BoardError;
use aether_core::{BitBoard, Color, File, MoveState, Rank, Square, attackers_to_square};
use cache::BoardCache;
use game_state::GameState;
use std::num::NonZeroU64;

pub type Result<T> = std::result::Result<T, BoardError>;

#[derive(Debug, Clone)]
pub struct Board {
    pieces: [[BitBoard; 6]; 2],
    game_state: GameState,
    cache: BoardCache,
    zobrist_hash: u64,
    /// Stack to store move states for unmake operations
    #[allow(dead_code)]
    move_history: Vec<MoveState>,
}

impl Board {
    /// Creates a new, empty board
    pub fn new() -> Result<Self> {
        BoardBuilder::new().build()
    }

    /// Creates a board set up in the standard starting position
    pub fn starting_position() -> Result<Self> {
        BoardBuilder::starting_position().build()
    }

    /// Returns a builder for constructing a custom board position
    pub fn builder() -> BoardBuilder {
        BoardBuilder::new()
    }

    /// Returns the piece bitboards for both colors and all piece types
    pub fn pieces(&self) -> &[[BitBoard; 6]; 2] {
        &self.pieces
    }

    /// Returns the current game state
    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    /// Returns the Zobrist hash of the current position, if non-zero
    pub fn zobrist_hash(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.zobrist_hash)
    }

    /// Invalidates cached data (e.g., check status)
    pub fn invalidate_cache(&mut self) {
        self.cache.invalidate_check_cache();
    }

    /// Changes the side to move and invalidates relevant caches
    pub fn change_side_to_move(&mut self) {
        self.game_state.side_to_move = self.game_state.side_to_move.opponent();
        self.invalidate_cache();
    }

    /// Returns a BitBoard of all pieces of the given color attacking the specified square
    pub fn attackers_to_square(&self, sq: Square, color: Color) -> BitBoard {
        attackers_to_square(sq, color, self.cache.occupied, &self.pieces[color as usize])
    }

    /// Prints an ASCII diagram of the current board to stdout.
    pub fn print(&self) {
        println!("{}", self.as_ascii());
    }

    /// Generates an ASCII diagram of the current board position.
    pub fn as_ascii(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        write!(out, "\n").unwrap();
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
            write!(out, "\n").unwrap();
        }
        writeln!(out, "  A B C D E F G H").unwrap();
        out
    }
}

impl Default for Board {
    /// Creates a board in the standard starting position
    fn default() -> Self {
        Self::starting_position().expect("Failed to create starting position")
    }
}
