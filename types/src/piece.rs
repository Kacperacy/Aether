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
