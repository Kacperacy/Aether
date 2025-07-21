use crate::{cache::BoardCache, error::*, game_state::GameState};
use aether_types::{BitBoard, Color, File, Piece, Square};

pub struct BoardBuilder {
    pieces: [[BitBoard; 6]; 2],
    game_state: GameState,
}

impl BoardBuilder {
    pub fn new() -> Self {
        Self {
            pieces: [[BitBoard::EMPTY; 6]; 2],
            game_state: GameState::new(),
        }
    }

    pub fn starting_position() -> Self {
        let mut builder = Self::new();
        builder.game_state = GameState::starting_position();
        builder.setup_starting_pieces();
        builder
    }

    pub fn place_piece(mut self, square: Square, piece: Piece, color: Color) -> Result<Self> {
        if self.is_square_occupied(square) {
            return Err(BoardError::OverlappingPieces { square });
        }

        self.pieces[color as usize][piece as usize] |= square.bitboard();
        Ok(self)
    }

    pub fn set_side_to_move(mut self, color: Color) -> Self {
        self.game_state.side_to_move = color;
        self
    }

    pub fn set_castling_rights(
        mut self,
        color: Color,
        rights: aether_types::CastlingRights,
    ) -> Self {
        self.game_state.castling_rights[color as usize] = rights;
        self
    }

    pub fn set_en_passant(mut self, square: Option<Square>) -> Result<Self> {
        if let Some(sq) = square {
            // Validate en passant square
            let expected_rank = match self.game_state.side_to_move {
                Color::White => aether_types::Rank::Six,
                Color::Black => aether_types::Rank::Three,
            };
            if sq.rank() != expected_rank {
                return Err(BoardError::InvalidEnPassantSquare { square: sq });
            }
        }
        self.game_state.en_passant_square = square;
        Ok(self)
    }

    pub fn build(self) -> Result<super::Board> {
        self.validate()?;

        let mut cache = BoardCache::new();
        cache.refresh(&self.pieces);

        let zobrist_hash = self.compute_zobrist_hash();

        Ok(super::Board {
            pieces: self.pieces,
            game_state: self.game_state,
            cache,
            zobrist_hash,
        })
    }

    fn validate(&self) -> Result<()> {
        // Check for exactly one king per side
        for color in [Color::White, Color::Black] {
            let king_count = self.pieces[color as usize][Piece::King as usize].len();
            match king_count {
                0 => return Err(BoardError::KingNotFound { color }),
                1 => {}
                _ => return Err(BoardError::MultipleKings { color }),
            }
        }

        // Validate castling rights
        self.validate_castling_rights()?;

        Ok(())
    }

    fn validate_castling_rights(&self) -> Result<()> {
        for &color in &[Color::White, Color::Black] {
            let rights = &self.game_state.castling_rights[color as usize];
            if rights.is_empty() {
                continue;
            }
            let king_square = Square::new(File::E, color.back_rank());
            if !self.pieces[color as usize][Piece::King as usize].has(king_square) {
                return Err(BoardError::InvalidCastlingRights {
                    reason: format!("{color} king not on starting square"),
                });
            }
        }
        Ok(())
    }

    fn setup_starting_pieces(&mut self) {
        // White pieces
        self.pieces[Color::White as usize][Piece::Pawn as usize] = BitBoard(0x000000000000FF00);
        self.pieces[Color::White as usize][Piece::Rook as usize] = BitBoard(0x0000000000000081);
        self.pieces[Color::White as usize][Piece::Knight as usize] = BitBoard(0x0000000000000042);
        self.pieces[Color::White as usize][Piece::Bishop as usize] = BitBoard(0x0000000000000024);
        self.pieces[Color::White as usize][Piece::Queen as usize] = BitBoard(0x0000000000000008);
        self.pieces[Color::White as usize][Piece::King as usize] = BitBoard(0x0000000000000010);

        // Black pieces
        self.pieces[Color::Black as usize][Piece::Pawn as usize] = BitBoard(0x00FF000000000000);
        self.pieces[Color::Black as usize][Piece::Rook as usize] = BitBoard(0x8100000000000000);
        self.pieces[Color::Black as usize][Piece::Knight as usize] = BitBoard(0x4200000000000000);
        self.pieces[Color::Black as usize][Piece::Bishop as usize] = BitBoard(0x2400000000000000);
        self.pieces[Color::Black as usize][Piece::Queen as usize] = BitBoard(0x0800000000000000);
        self.pieces[Color::Black as usize][Piece::King as usize] = BitBoard(0x1000000000000000);
    }

    fn is_square_occupied(&self, square: Square) -> bool {
        for color in 0..2 {
            for piece in 0..6 {
                if self.pieces[color][piece].has(square) {
                    return true;
                }
            }
        }
        false
    }

    fn compute_zobrist_hash(&self) -> u64 {
        // Placeholder for zobrist hash computation
        // In full implementation, this would use proper zobrist tables
        0
    }
}

impl Default for BoardBuilder {
    fn default() -> Self {
        Self::new()
    }
}
