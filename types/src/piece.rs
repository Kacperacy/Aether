use std::str::FromStr;

/// Represents a chess piece type.
///
/// Each variant has an associated material value in centipawns (100 = 1 pawn).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Piece {
    /// Pawn (value: 100)
    Pawn = 0,
    /// Knight (value: 320)
    Knight = 1,
    /// Bishop (value: 330)
    Bishop = 2,
    /// Rook (value: 500)
    Rook = 3,
    /// Queen (value: 900)
    Queen = 4,
    /// King (value: 20000 - effectively priceless)
    King = 5,
}

impl FromStr for Piece {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "p" => Ok(Self::Pawn),
            "n" => Ok(Self::Knight),
            "b" => Ok(Self::Bishop),
            "r" => Ok(Self::Rook),
            "q" => Ok(Self::Queen),
            "k" => Ok(Self::King),
            _ => Err(()),
        }
    }
}

impl Piece {
    /// Number of piece types (6)
    pub const NUM: usize = 6;

    /// Converts piece to a single character (lowercase).
    ///
    /// Returns 'p', 'n', 'b', 'r', 'q', or 'k'.
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

    /// Creates a piece from a character (lowercase).
    ///
    /// Accepts 'p', 'n', 'b', 'r', 'q', or 'k'.
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

    /// Returns an array containing all piece types.
    pub const fn all() -> [Self; 6] {
        [
            Self::Pawn,
            Self::Knight,
            Self::Bishop,
            Self::Rook,
            Self::Queen,
            Self::King,
        ]
    }

    /// Returns the material value of the piece in centipawns.
    ///
    /// Values: Pawn=100, Knight=320, Bishop=330, Rook=500, Queen=900, King=20000
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

    /// Returns `true` if this piece slides (Bishop, Rook, or Queen).
    pub const fn is_sliding(self) -> bool {
        matches!(self, Self::Bishop | Self::Rook | Self::Queen)
    }

    /// Returns `true` if this piece is a major piece (Rook or Queen).
    pub const fn is_major(self) -> bool {
        matches!(self, Self::Rook | Self::Queen)
    }

    /// Returns `true` if this piece is a minor piece (Knight or Bishop).
    pub const fn is_minor(self) -> bool {
        matches!(self, Self::Knight | Self::Bishop)
    }
}
