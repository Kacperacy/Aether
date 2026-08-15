use crate::error::BoardError::{
    InvalidCastlingRights, InvalidEnPassantSquare, KingNotFound, MultipleKings, OverlappingPieces,
};
use crate::state_info::StateInfo;
use crate::{MAX_SEARCH_DEPTH, Result, ZOBRIST_HISTORY_CAPACITY, cache::BoardCache};
use aether_core::{BitBoard, CastlingRights, Color, File, Piece, Square};

pub struct BoardBuilder {
    pieces: [[BitBoard; Piece::NUM]; Color::NUM],
    side_to_move: Color,
    fullmove_number: u16,
    castling_rights: CastlingRights,
    en_passant_square: Option<Square>,
    halfmove_clock: u16,
}

impl BoardBuilder {
    pub fn new() -> Self {
        Self {
            pieces: [[BitBoard::EMPTY; Piece::NUM]; Color::NUM],
            side_to_move: Color::White,
            fullmove_number: 1,
            castling_rights: CastlingRights::NONE,
            en_passant_square: None,
            halfmove_clock: 0,
        }
    }

    pub fn starting_position() -> Self {
        let mut builder = Self::new();
        builder.castling_rights = CastlingRights::ALL;
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

    pub fn set_castling_rights(&mut self, rights: CastlingRights) -> &mut Self {
        self.castling_rights = rights;
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

        let mut mailbox = [None; Square::NUM];
        for color in Color::ALL {
            for &piece in &Piece::ALL {
                for square in self.pieces[color as usize][piece as usize].iter() {
                    mailbox[square.to_index() as usize] = Some((piece, color));
                }
            }
        }

        let white_king_sq = self.pieces[Color::White as usize][Piece::King as usize].lsb();
        let black_king_sq = self.pieces[Color::Black as usize][Piece::King as usize].lsb();

        let stm = self.side_to_move;
        let king_sq = if stm == Color::White {
            white_king_sq
        } else {
            black_king_sq
        };
        let checkers = attacks::attackers_to_square(
            king_sq,
            stm.opponent(),
            cache.occupied,
            &self.pieces[stm.opponent() as usize],
        );

        let white_occ = cache.color_combined[Color::White as usize];
        let black_occ = cache.color_combined[Color::Black as usize];

        let (white_blockers, white_pinners) = attacks::compute_slider_blockers(
            white_king_sq,
            white_occ,
            &self.pieces[Color::Black as usize],
            cache.occupied,
        );
        let (black_blockers, black_pinners) = attacks::compute_slider_blockers(
            black_king_sq,
            black_occ,
            &self.pieces[Color::White as usize],
            cache.occupied,
        );

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
                king_square: [white_king_sq, black_king_sq],
                checkers,
                blockers_for_king: [white_blockers, black_blockers],
                pinners: [white_pinners, black_pinners],
            },
            state_history: [StateInfo::default(); MAX_SEARCH_DEPTH],
            history_index: 0,
            zobrist_history: Vec::with_capacity(ZOBRIST_HISTORY_CAPACITY),
        };

        board.state.zobrist_hash = board.calculate_zobrist_hash();

        Ok(board)
    }

    fn validate(&self) -> Result<()> {
        for color in Color::ALL {
            let king_count = self.pieces[color as usize][Piece::King as usize].count();
            match king_count {
                0 => return Err(KingNotFound { color }),
                1 => {}
                _ => return Err(MultipleKings { color }),
            }
        }

        self.validate_castling_rights()?;

        Ok(())
    }

    fn validate_castling_rights(&self) -> Result<()> {
        for color in Color::ALL {
            if !self.castling_rights.any(CastlingRights::for_color(color)) {
                continue;
            }
            let king_square = Square::new(File::E, color.back_rank());
            if !self.pieces[color as usize][Piece::King as usize].contains(king_square) {
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

    /// The builder has no occupancy cache yet, so fold the piece boards.
    fn is_square_occupied(&self, square: Square) -> bool {
        let bb = square.bitboard();
        self.pieces
            .iter()
            .flatten()
            .any(|pieces| !(*pieces & bb).is_empty())
    }
}

impl Default for BoardBuilder {
    fn default() -> Self {
        Self::new()
    }
}
