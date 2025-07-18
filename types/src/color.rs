use std::fmt::Display;
use std::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl Color {
    pub const NUM: usize = 2;

    pub const fn as_char(self) -> char {
        match self {
            Self::White => 'w',
            Self::Black => 'b',
        }
    }

    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'w' => Some(Self::White),
            'b' => Some(Self::Black),
            _ => None,
        }
    }

    pub const fn all() -> [Self; 2] {
        [Self::White, Self::Black]
    }

    pub const fn opponent(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    pub const fn pawn_start_rank(self) -> crate::Rank {
        match self {
            Self::White => crate::Rank::Two,
            Self::Black => crate::Rank::Seven,
        }
    }

    pub const fn pawn_promotion_rank(self) -> crate::Rank {
        match self {
            Self::White => crate::Rank::Eight,
            Self::Black => crate::Rank::One,
        }
    }

    pub const fn forward_direction(self) -> i8 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }
}
