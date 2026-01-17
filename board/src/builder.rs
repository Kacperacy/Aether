use crate::error::BoardError::{
    InvalidCastlingRights, InvalidEnPassantSquare, KingNotFound, MultipleKings, OverlappingPieces,
};
use crate::state_info::StateInfo;
use crate::{
    MAX_GAME_PHASE, MAX_SEARCH_DEPTH, PHASE_BISHOP, PHASE_KNIGHT, PHASE_QUEEN, PHASE_ROOK,
    PHASE_TOTAL, Result, ZOBRIST_HISTORY_CAPACITY, cache::BoardCache, pst,
};
use aether_core::{ALL_COLORS, ALL_PIECES, BitBoard, CastlingRights, Color, File, Piece, Square};

pub struct BoardBuilder {
    pieces: [[BitBoard; Piece::NUM]; Color::NUM],
    side_to_move: Color,
    fullmove_number: u16,
    castling_rights: [CastlingRights; 2],
    en_passant_square: Option<Square>,
    halfmove_clock: u16,
}

impl BoardBuilder {
    pub fn new() -> Self {
        Self {
            pieces: [[BitBoard::EMPTY; Piece::NUM]; Color::NUM],
            side_to_move: Color::White,
            fullmove_number: 1,
            castling_rights: [CastlingRights::EMPTY; 2],
            en_passant_square: None,
            halfmove_clock: 0,
        }
    }

    pub fn starting_position() -> Self {
        let mut builder = Self::new();
        builder.castling_rights = [
            CastlingRights {
                short: Some(File::H),
                long: Some(File::A),
            },
            CastlingRights {
                short: Some(File::H),
                long: Some(File::A),
            },
        ];
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
        self.side_to_move = color;
        self
    }

    pub fn set_castling_rights(&mut self, color: Color, rights: CastlingRights) -> &mut Self {
        self.castling_rights[color as usize] = rights;
        self
    }

    pub fn set_en_passant(&mut self, square: Option<Square>) -> Result<&mut Self> {
        if let Some(sq) = square {
            let expected_rank = self.side_to_move.en_passant_rank();
            if sq.rank() != expected_rank {
                return Err(InvalidEnPassantSquare { square: sq });
            }
        }
        self.en_passant_square = square;
        Ok(self)
    }

    pub fn set_halfmove_clock(&mut self, clock: u16) -> &mut Self {
        self.halfmove_clock = clock;
        self
    }

    pub fn set_fullmove_number(&mut self, number: u16) -> &mut Self {
        self.fullmove_number = if number == 0 { 1 } else { number };
        self
    }

    pub fn build(self) -> Result<super::Board> {
        self.validate()?;

        let mut cache = BoardCache::new();
        cache.refresh(&self.pieces);

        let game_phase = self.compute_game_phase();
        let (pst_mg, pst_eg) = pst::compute_pst_score(&self.pieces);

        let mut mailbox = [None; Square::NUM];
        for color in ALL_COLORS {
            for &piece in &ALL_PIECES {
                for square in self.pieces[color as usize][piece as usize].iter() {
                    mailbox[square.to_index() as usize] = Some((piece, color));
                }
            }
        }

        let mut board = super::Board {
            pieces: self.pieces,
            mailbox,
            cache,
            side_to_move: self.side_to_move,
            fullmove_number: self.fullmove_number,
            state: StateInfo {
                castling_rights: self.castling_rights,
                en_passant_square: self.en_passant_square,
                halfmove_clock: self.halfmove_clock,
                captured_piece: None,
                zobrist_hash: 0,
                game_phase,
                pst_mg,
                pst_eg,
            },
            state_history: [StateInfo::default(); MAX_SEARCH_DEPTH],
            history_index: 0,
            zobrist_history: Vec::with_capacity(ZOBRIST_HISTORY_CAPACITY),
        };

        board.state.zobrist_hash = board.calculate_zobrist_hash();

        Ok(board)
    }

    fn compute_game_phase(&self) -> i16 {
        let knights = (self.pieces[Color::White as usize][Piece::Knight as usize].count()
            + self.pieces[Color::Black as usize][Piece::Knight as usize].count())
            as i32;
        let bishops = (self.pieces[Color::White as usize][Piece::Bishop as usize].count()
            + self.pieces[Color::Black as usize][Piece::Bishop as usize].count())
            as i32;
        let rooks = (self.pieces[Color::White as usize][Piece::Rook as usize].count()
            + self.pieces[Color::Black as usize][Piece::Rook as usize].count())
            as i32;
        let queens = (self.pieces[Color::White as usize][Piece::Queen as usize].count()
            + self.pieces[Color::Black as usize][Piece::Queen as usize].count())
            as i32;

        let material = knights * PHASE_KNIGHT as i32
            + bishops * PHASE_BISHOP as i32
            + rooks * PHASE_ROOK as i32
            + queens * PHASE_QUEEN as i32;

        ((material * MAX_GAME_PHASE) / PHASE_TOTAL as i32).min(MAX_GAME_PHASE) as i16
    }

    fn validate(&self) -> Result<()> {
        // Check for exactly one king per side
        for color in ALL_COLORS {
            let king_count = self.pieces[color as usize][Piece::King as usize].count();
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
            let rights = &self.castling_rights[color as usize];
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
        for color in ALL_COLORS {
            for &piece in &ALL_PIECES {
                if self.pieces[color as usize][piece as usize].has(square) {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for BoardBuilder {
    fn default() -> Self {
        Self::new()
    }
}