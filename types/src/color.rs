use std::ops::Not;

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
