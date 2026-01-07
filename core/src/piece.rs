use crate::TypeError::InvalidPiece;
use crate::{Result, Score, TypeError};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Piece {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

pub const ALL_PIECES: [Piece; Piece::NUM] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

pub const PROMOTION_PIECES: [Piece; 4] = [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];
pub const PAWN_VALUE: Score = 100;
pub const KNIGHT_VALUE: Score = 320;
pub const BISHOP_VALUE: Score = 330;
pub const ROOK_VALUE: Score = 500;
pub const QUEEN_VALUE: Score = 900;
pub const KING_VALUE: Score = 20000;

pub const PIECE_VALUES: [Score; Piece::NUM] = [
    PAWN_VALUE,
    KNIGHT_VALUE,
    BISHOP_VALUE,
    ROOK_VALUE,
    QUEEN_VALUE,
    KING_VALUE,
];

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl FromStr for Piece {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "p" => Ok(Self::Pawn),
            "n" => Ok(Self::Knight),
            "b" => Ok(Self::Bishop),
            "r" => Ok(Self::Rook),
            "q" => Ok(Self::Queen),
            "k" => Ok(Self::King),
            _ => Err(InvalidPiece {
                piece: s.to_string(),
            }),
        }
    }
}

impl Piece {
    pub const NUM: usize = 6;

    pub const fn as_char(self) -> char {
        match self {
            Self::Pawn => 'p',
            Self::Knight => 'n',
            Self::Bishop => 'b',
            Self::Rook => 'r',
            Self::Queen => 'q',
            Self::King => 'k',
        }
    }

    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'p' => Some(Self::Pawn),
            'n' => Some(Self::Knight),
            'b' => Some(Self::Bishop),
            'r' => Some(Self::Rook),
            'q' => Some(Self::Queen),
            'k' => Some(Self::King),
            _ => None,
        }
    }

    pub const fn value(self) -> Score {
        match self {
            Self::Pawn => PAWN_VALUE,
            Self::Knight => KNIGHT_VALUE,
            Self::Bishop => BISHOP_VALUE,
            Self::Rook => ROOK_VALUE,
            Self::Queen => QUEEN_VALUE,
            Self::King => KING_VALUE,
        }
    }

    pub const fn is_sliding(self) -> bool {
        matches!(self, Self::Bishop | Self::Rook | Self::Queen)
    }

    pub const fn is_major(self) -> bool {
        matches!(self, Self::Rook | Self::Queen)
    }

    pub const fn is_minor(self) -> bool {
        matches!(self, Self::Knight | Self::Bishop)
    }
}
