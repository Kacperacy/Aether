mod builder;
mod cache;
mod check;
mod error;
mod fen;
mod game_state;
mod movement;
mod zobrist;

pub use builder::BoardBuilder;
pub use cache::BoardCache;
pub use error::*;
pub use game_state::GameState;

use aether_types::{BitBoard, BoardQuery, Color, File, Piece, Rank, Square};
use std::num::NonZeroU64;

#[derive(Debug, Clone)]
pub struct Board {
    pieces: [[BitBoard; 6]; 2],
    game_state: GameState,
    cache: BoardCache,
    zobrist_hash: u64,
}

impl Board {
    pub fn new() -> Result<Self> {
        BoardBuilder::new().build()
    }

    pub fn starting_position() -> Result<Self> {
        BoardBuilder::starting_position().build()
    }

    pub fn builder() -> BoardBuilder {
        BoardBuilder::new()
    }

    pub fn pieces(&self) -> &[[BitBoard; 6]; 2] {
        &self.pieces
    }

    pub fn game_state(&self) -> &GameState {
        &self.game_state
    }

    pub fn zobrist_hash(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.zobrist_hash)
    }

    pub fn update_zobrist_incremental(
        &mut self,
        _piece: Piece,
        _color: Color,
        _from: Square,
        _to: Square,
    ) {
        // Incremental zobrist update - remove from old square, add to new square
        // self.zobrist_hash ^= zobrist::piece_hash(piece, color, from);
        // self.zobrist_hash ^= zobrist::piece_hash(piece, color, to);

        // not implemented yet
        self.zobrist_hash = 0; // Placeholder for now
    }

    fn update_cache(&mut self) {
        self.cache.refresh(&self.pieces);
    }

    pub fn invalidate_cache(&mut self) {
        self.cache.invalidate_check_cache();
    }
}

impl BoardQuery for Board {
    fn piece_at(&self, square: Square) -> Option<(Piece, Color)> {
        if !self.cache.occupied.has(square) {
            return None;
        }

        let color = if self.cache.color_combined[Color::White as usize].has(square) {
            Color::White
        } else {
            Color::Black
        };

        Piece::all()
            .into_iter()
            .find(|&p| self.pieces[color as usize][p as usize].has(square))
            .map(|p| (p, color))
    }

    fn is_square_occupied(&self, square: Square) -> bool {
        self.cache.occupied.has(square)
    }

    fn is_square_attacked(&self, square: Square, by_color: Color) -> bool {
        self.attackers_to_square(square, by_color).is_empty()
    }

    fn get_king_square(&self, color: Color) -> Option<Square> {
        self.pieces[color as usize][Piece::King as usize].to_square()
    }

    fn can_castle_short(&self, color: Color) -> bool {
        self.game_state.castling_rights[color as usize]
            .short
            .is_some()
    }

    fn can_castle_long(&self, color: Color) -> bool {
        self.game_state.castling_rights[color as usize]
            .long
            .is_some()
    }

    fn en_passant_square(&self) -> Option<Square> {
        self.game_state.en_passant_square
    }

    fn side_to_move(&self) -> Color {
        self.game_state.side_to_move
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::starting_position().expect("Failed to create starting position")
    }
}

// Trait for board operations
pub trait BoardOps {
    fn make_move(&mut self, mv: aether_types::Move) -> Result<()>;
    fn unmake_move(&mut self, mv: aether_types::Move) -> Result<()>;
}

// Trait for mutable board operations
pub trait BoardMut {
    fn set_piece(&mut self, square: Square, piece: Piece, color: Color);
    fn remove_piece(&mut self, square: Square);
    fn clear_board(&mut self);
}

impl BoardMut for Board {
    fn set_piece(&mut self, square: Square, piece: Piece, color: Color) {
        self.pieces[color as usize][piece as usize] |= square.bitboard();
        self.update_cache();
        self.invalidate_cache();
    }

    fn remove_piece(&mut self, square: Square) {
        for color in 0..2 {
            for piece in 0..6 {
                self.pieces[color][piece] &= !square.bitboard();
            }
        }
        self.update_cache();
        self.invalidate_cache();
    }

    fn clear_board(&mut self) {
        self.pieces = [[BitBoard::EMPTY; 6]; 2];
        self.update_cache();
        self.invalidate_cache();
    }
}

// Extension methods for board
impl Board {
    pub fn print(&self) {
        println!("{}", self.as_ascii());
    }

    /// Returns an ASCII diagram of the current board.
    pub fn as_ascii(&self) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        writeln!(out, " +---+---+---+---+---+---+---+---+").unwrap();
        for rank in (0..8).rev() {
            write!(out, "{} |", rank + 1).unwrap();
            for file in 0..8 {
                let sq = Square::new(File::from_index(file), Rank::new(rank));
                let ch = self.piece_at(sq).map_or('.', |(p, c)| {
                    let ch = p.as_char();
                    if c == Color::White {
                        ch.to_ascii_uppercase()
                    } else {
                        ch
                    }
                });
                write!(out, " {} |", ch).unwrap();
            }
            writeln!(out, "\n +---+---+---+---+---+---+---+---+").unwrap();
        }
        writeln!(out, "   a   b   c   d   e   f   g   h").unwrap();
        out
    }
}
