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

#[derive(Debug, Clone)]
pub struct Board {
    pieces: [[BitBoard; 6]; 2],
    game_state: GameState,
    cache: BoardCache,
    zobrist_hash: u64,
    /// Stack to store move states for unmake operations
    move_history: Vec<MoveState>,
}

impl Board {
    /// Creates a new, empty board (no pieces, white to move)
    pub fn empty() -> Self {
        Self {
            pieces: [[BitBoard::EMPTY; 6]; 2],
            game_state: GameState::new(),
            cache: BoardCache::new(),
            zobrist_hash: 0,
            move_history: Vec::new(),
        }
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
    #[inline]
    pub fn pieces(&self) -> &[[BitBoard; 6]; 2] {
        &self.pieces
    }

    /// Returns the current game state
    #[inline]
    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    /// Returns mutable reference to game state (for internal use)
    #[inline]
    pub(crate) fn game_state_mut(&mut self) -> &mut GameState {
        &mut self.game_state
    }

    /// Returns the Zobrist hash of the current position, if non-zero
    #[inline]
    pub fn zobrist_hash(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.zobrist_hash)
    }

    /// Returns the raw zobrist hash value
    #[inline]
    pub fn zobrist_hash_raw(&self) -> u64 {
        self.zobrist_hash
    }

    /// Sets the zobrist hash (for internal use)
    #[inline]
    pub(crate) fn set_zobrist_hash(&mut self, hash: u64) {
        self.zobrist_hash = hash;
    }

    /// Returns reference to move history
    #[inline]
    pub fn move_history(&self) -> &[MoveState] {
        &self.move_history
    }

    /// Returns mutable reference to move history (for internal use)
    #[inline]
    pub(crate) fn move_history_mut(&mut self) -> &mut Vec<MoveState> {
        &mut self.move_history
    }

    /// Returns reference to the cache
    #[inline]
    pub fn cache(&self) -> &BoardCache {
        &self.cache
    }

    /// Refreshes the board cache from current piece positions
    #[inline]
    pub fn refresh_cache(&mut self) {
        self.cache.refresh(&self.pieces);
    }

    /// Returns a BitBoard of all pieces of the given color attacking the specified square
    #[inline]
    pub fn attackers_to_square(&self, sq: Square, color: Color) -> BitBoard {
        attackers_to_square(sq, color, self.cache.occupied, &self.pieces[color as usize])
    }

    /// Returns the occupied squares bitboard
    #[inline]
    pub fn occupied(&self) -> BitBoard {
        self.cache.occupied
    }

    /// Returns the bitboard of all pieces of a given color
    #[inline]
    pub fn color_occupied(&self, color: Color) -> BitBoard {
        self.cache.color_combined[color as usize]
    }

    /// Prints an ASCII diagram of the current board to stdout.
    pub fn print(&self) {
        println!("{}", self.as_ascii());
    }

    /// Generates an ASCII diagram of the current board position.
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

    /// Returns the number of half-moves played
    #[inline]
    pub fn ply(&self) -> usize {
        self.move_history.len()
    }

    /// Checks if the position is a draw by fifty-move rule
    #[inline]
    pub fn is_fifty_move_draw(&self) -> bool {
        self.game_state.halfmove_clock >= 100
    }

    /// Piece and color at square, if any. (Delegates to BoardQuery)
    #[inline]
    pub fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        <Self as BoardQuery>::piece_at(self, square)
    }
}

impl Default for Board {
    /// Creates a board in the standard starting position
    fn default() -> Self {
        Self::starting_position().expect("Failed to create starting position")
    }
}
