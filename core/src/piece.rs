use crate::CoreError::InvalidPiece;
use crate::{CoreError, Result};
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

impl Piece {
    pub const NUM: usize = 6;

    pub const ALL: [Self; Self::NUM] = [
        Self::Pawn,
        Self::Knight,
        Self::Bishop,
        Self::Rook,
        Self::Queen,
        Self::King,
    ];

    pub const PROMOTIONS: [Self; 4] = [Self::Knight, Self::Bishop, Self::Rook, Self::Queen];

    #[inline(always)]
    pub const fn from_index(index: u8) -> Self {
        debug_assert!(index < 6, "Piece index out of bounds");
        match index {
            0 => Self::Pawn,
            1 => Self::Knight,
            2 => Self::Bishop,
            3 => Self::Rook,
            4 => Self::Queen,
            _ => Self::King,
        }
    }

    #[inline(always)]
    pub const fn to_index(self) -> u8 {
        self as u8
    }

    #[inline(always)]
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

    #[inline(always)]
    pub const fn from_char(c: char) -> Option<Self> {
        match c {
            'p' | 'P' => Some(Self::Pawn),
            'n' | 'N' => Some(Self::Knight),
            'b' | 'B' => Some(Self::Bishop),
            'r' | 'R' => Some(Self::Rook),
            'q' | 'Q' => Some(Self::Queen),
            'k' | 'K' => Some(Self::King),
            _ => None,
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl FromStr for Piece {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self> {
        let chars: Vec<char> = s.chars().collect();
        if chars.len() == 1
            && let Some(piece) = Self::from_char(chars[0])
        {
            return Ok(piece);
        }

        Err(InvalidPiece {
            piece: s.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_creation_and_index() {
        assert_eq!(Piece::Pawn.to_index(), 0);
        assert_eq!(Piece::King.to_index(), 5);

        assert_eq!(Piece::from_index(1), Piece::Knight);
        assert_eq!(Piece::from_index(4), Piece::Queen);
    }

    #[test]
    fn test_as_char_and_from_char() {
        assert_eq!(Piece::Knight.as_char(), 'n');
        assert_eq!(Piece::King.as_char(), 'k');

        assert_eq!(Piece::from_char('Q'), Some(Piece::Queen));
        assert_eq!(Piece::from_char('p'), Some(Piece::Pawn));
        assert_eq!(Piece::from_char('x'), None);
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Piece::from_str("r").unwrap(), Piece::Rook);
        assert_eq!(Piece::from_str("B").unwrap(), Piece::Bishop);

        assert!(Piece::from_str("x").is_err());
        assert!(Piece::from_str("Knight").is_err());
        assert!(Piece::from_str("").is_err());
    }
}
