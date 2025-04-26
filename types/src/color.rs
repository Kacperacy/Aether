use std::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
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
}
