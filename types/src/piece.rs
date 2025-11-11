use crate::TypeError::InvalidPiece;
use crate::{TypeError, TypeResult};
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

/// All piece types in chess
pub const ALL_PIECES: [Piece; Piece::NUM] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

impl FromStr for Piece {
    type Err = TypeError;

    fn from_str(s: &str) -> TypeResult<Self> {
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
    /// Number of piece types in chess
    pub const NUM: usize = 6;

    /// Returns the character representation of the piece (lowercase)
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

    /// Creates a Piece from its character representation (lowercase)
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

    /// Returns the standard material value of the piece in centipawns
    pub const fn value(self) -> u16 {
        match self {
            Self::Pawn => 100,
            Self::Knight => 320,
            Self::Bishop => 330,
            Self::Rook => 500,
            Self::Queen => 900,
            Self::King => 20000,
        }
    }

    /// Returns true if the piece is a sliding piece (Bishop, Rook, Queen)
    pub const fn is_sliding(self) -> bool {
        matches!(self, Self::Bishop | Self::Rook | Self::Queen)
    }

    /// Returns true if the piece is a major piece (Rook, Queen)
    pub const fn is_major(self) -> bool {
        matches!(self, Self::Rook | Self::Queen)
    }

    /// Returns true if the piece is a minor piece (Knight, Bishop)
    pub const fn is_minor(self) -> bool {
        matches!(self, Self::Knight | Self::Bishop)
    }
}
