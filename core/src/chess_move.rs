use crate::{CastlingRights, Color, Piece, Square};
use std::fmt::{self, Display, Formatter};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub capture: Option<Piece>,
    pub promotion: Option<Piece>,
    pub flags: MoveFlags,
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;

        if let Some(p) = self.promotion {
            let symbol = match p {
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                Piece::Queen => 'q',
                // King and Pawn can never appear here – unreachable by rule
                _ => unreachable!("illegal promotion piece"),
            };
            write!(f, "{symbol}")?;
        }

        Ok(())
    }
}

impl Move {
    pub fn new(from: Square, to: Square, piece: Piece) -> Self {
        Move {
            from,
            to,
            piece,
            capture: None,
            promotion: None,
            flags: MoveFlags {
                is_castle: false,
                is_en_passant: false,
                is_double_pawn_push: false,
            },
        }
    }

    pub const fn with_promotion(mut self, piece: Piece) -> Self {
        self.promotion = Some(piece);
        self
    }

    pub const fn with_piece(mut self, piece: Piece) -> Self {
        self.piece = piece;
        self
    }

    pub const fn with_capture(mut self, piece: Piece) -> Self {
        self.capture = Some(piece);
        self
    }

    pub const fn with_flags(mut self, flags: MoveFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub const fn is_capture(&self) -> bool {
        self.capture.is_some()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MoveFlags {
    pub is_castle: bool,
    pub is_en_passant: bool,
    pub is_double_pawn_push: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MoveState {
    pub captured_piece: Option<(Piece, Color)>,
    pub mv_from: Square,
    pub mv_to: Square,
    pub promotion: Option<Piece>,

    /* game-state members */
    pub old_zobrist_hash: u64,
    pub old_en_passant: Option<Square>,
    pub old_castling_rights: [CastlingRights; 2], // [color][side]
    pub old_halfmove_clock: u16,
    pub old_game_phase: i16,
    pub old_pst_mg: i32,
    pub old_pst_eg: i32,
}

impl Default for MoveState {
    fn default() -> Self {
        MoveState {
            captured_piece: None,
            mv_from: Square::A1,
            mv_to: Square::A1,
            promotion: None,
            old_zobrist_hash: 0,
            old_en_passant: None,
            old_castling_rights: [CastlingRights::EMPTY; 2],
            old_halfmove_clock: 0,
            old_game_phase: 0,
            old_pst_mg: 0,
            old_pst_eg: 0,
        }
    }
}

impl Default for Move {
    fn default() -> Self {
        Move {
            from: Square::A1,
            to: Square::A1,
            piece: Piece::Pawn,
            capture: None,
            promotion: None,
            flags: MoveFlags::default(),
        }
    }
}
