use crate::TypeError::{InvalidRank, InvalidRankIndex};
use crate::{BitBoard, Color, Result, TypeError};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

/// All ranks on a chessboard
pub const ALL_RANKS: [Rank; 8] = [
    Rank::One,
    Rank::Two,
    Rank::Three,
    Rank::Four,
    Rank::Five,
    Rank::Six,
    Rank::Seven,
    Rank::Eight,
];

impl FromStr for Rank {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "1" => Ok(Self::One),
            "2" => Ok(Self::Two),
            "3" => Ok(Self::Three),
            "4" => Ok(Self::Four),
            "5" => Ok(Self::Five),
            "6" => Ok(Self::Six),
            "7" => Ok(Self::Seven),
            "8" => Ok(Self::Eight),
            _ => Err(InvalidRank(s.to_string())),
        }
    }
}

impl Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl Rank {
    /// Number of ranks on a chessboard
    pub const NUM: usize = 8;

    /// Safe conversion from index (0-7) to Rank
    pub fn try_from_index(rank: u8) -> Result<Self> {
        match rank {
            0 => Ok(Self::One),
            1 => Ok(Self::Two),
            2 => Ok(Self::Three),
            3 => Ok(Self::Four),
            4 => Ok(Self::Five),
            5 => Ok(Self::Six),
            6 => Ok(Self::Seven),
            7 => Ok(Self::Eight),
            _ => Err(InvalidRankIndex(rank)),
        }
    }

    /// Unsafe conversion from index (0-7) to Rank
    pub const fn from_index(rank: i8) -> Self {
        match rank {
            0 => Self::One,
            1 => Self::Two,
            2 => Self::Three,
            3 => Self::Four,
            4 => Self::Five,
            5 => Self::Six,
            6 => Self::Seven,
            7 => Self::Eight,
            _ => panic!("Invalid rank"),
        }
    }

    /// Returns the character representation of the Rank
    pub const fn as_char(self) -> char {
        match self {
            Self::One => '1',
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Seven => '7',
            Self::Eight => '8',
        }
    }

    /// Returns a new Rank offset by the given amount, or None if out of bounds
    pub const fn offset(self, offset: i8) -> Option<Self> {
        let new_rank = self as i8 + offset;
        if new_rank < 0 || new_rank > 7 {
            None
        } else {
            Some(Self::from_index(new_rank))
        }
    }

    /// Flips the rank vertically (mirrors across the horizontal axisr)
    pub const fn flip(self) -> Self {
        match self {
            Self::One => Self::Eight,
            Self::Two => Self::Seven,
            Self::Three => Self::Six,
            Self::Four => Self::Five,
            Self::Five => Self::Four,
            Self::Six => Self::Three,
            Self::Seven => Self::Two,
            Self::Eight => Self::One,
        }
    }

    /// Returns the bitboard representing all squares on this rank
    pub const fn bitboard(self) -> BitBoard {
        match self {
            Self::One => BitBoard(0x00000000000000ff),
            Self::Two => BitBoard(0x000000000000ff00),
            Self::Three => BitBoard(0x0000000000ff0000),
            Self::Four => BitBoard(0x00000000ff000000),
            Self::Five => BitBoard(0x000000ff00000000),
            Self::Six => BitBoard(0x0000ff0000000000),
            Self::Seven => BitBoard(0x00ff000000000000),
            Self::Eight => BitBoard(0xff00000000000000),
        }
    }

    /// Returns the rank relative to the given color (White's perspective)
    pub const fn relative_to(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => self.flip(),
        }
    }
}
