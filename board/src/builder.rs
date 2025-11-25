use crate::error::BoardError::{
    InvalidCastlingRights, InvalidEnPassantSquare, KingNotFound, MultipleKings, OverlappingPieces,
};
use crate::{Result, cache::BoardCache, game_state::GameState};
use aether_core::{ALL_COLORS, BitBoard, CastlingRights, Color, File, Piece, Rank, Square};

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

    pub fn place_piece(&mut self, square: Square, piece: Piece, color: Color) -> Result<&mut Self> {
        if self.is_square_occupied(square) {
            return Err(OverlappingPieces { square });
        }

        self.pieces[color as usize][piece as usize] |= square.bitboard();
        Ok(self)
    }

    pub fn set_side_to_move(&mut self, color: Color) -> &mut Self {
        self.game_state.side_to_move = color;
        self
    }

    pub fn set_castling_rights(&mut self, color: Color, rights: CastlingRights) -> &mut Self {
        self.game_state.castling_rights[color as usize] = rights;
        self
    }

    pub fn set_en_passant(&mut self, square: Option<Square>) -> Result<&mut Self> {
        if let Some(sq) = square {
            // Validate en passant square
            let expected_rank = match self.game_state.side_to_move {
                Color::White => Rank::Six,
                Color::Black => Rank::Three,
            };
            if sq.rank() != expected_rank {
                return Err(InvalidEnPassantSquare { square: sq });
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
            move_history: Vec::new(),
        })
    }

    fn validate(&self) -> Result<()> {
        // Check for exactly one king per side
        for color in ALL_COLORS {
            let king_count = self.pieces[color as usize][Piece::King as usize].len();
            match king_count {
                0 => return Err(KingNotFound { color }),
                1 => {}
                _ => return Err(MultipleKings { color }),
            }
        }

        // Validate castling rights
        self.validate_castling_rights()?;

        Ok(())
    }

    fn validate_castling_rights(&self) -> Result<()> {
        for color in ALL_COLORS {
            let rights = &self.game_state.castling_rights[color as usize];
            if rights.is_empty() {
                continue;
            }
            let king_square = Square::new(File::E, color.back_rank());
            if !self.pieces[color as usize][Piece::King as usize].has(king_square) {
                return Err(InvalidCastlingRights {
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
        use aether_core::{ALL_SQUARES, Piece, zobrist_keys};

        let keys = zobrist_keys();
        let mut hash = 0u64;

        // Hash pieces
        for &sq in &ALL_SQUARES {
            for color in ALL_COLORS {
                for piece_idx in 0..6 {
                    let piece = match piece_idx {
                        0 => Piece::Pawn,
                        1 => Piece::Knight,
                        2 => Piece::Bishop,
                        3 => Piece::Rook,
                        4 => Piece::Queen,
                        5 => Piece::King,
                        _ => continue,
                    };

                    if self.pieces[color as usize][piece_idx].has(sq) {
                        hash ^= keys.piece_key(sq, piece, color);
                    }
                }
            }
        }

        // Hash side to move
        if self.game_state.side_to_move == Color::Black {
            hash ^= keys.side_to_move;
        }

        // Hash castling rights
        for color in ALL_COLORS {
            let rights = &self.game_state.castling_rights[color as usize];
            if rights.short.is_some() {
                hash ^= keys.castling_key(color, true);
            }
            if rights.long.is_some() {
                hash ^= keys.castling_key(color, false);
            }
        }

        // Hash en passant
        if let Some(ep_sq) = self.game_state.en_passant_square {
            hash ^= keys.en_passant_key(ep_sq.file());
        }

        hash
    }
}

impl Default for BoardBuilder {
    fn default() -> Self {
        Self::new()
    }
}
