use crate::{BitBoard, Color, File, Rank, Result, TypeError};
use std::fmt::Display;
use std::str::FromStr;

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Square {
    A1 =  0, B1 =  1, C1 =  2, D1 =  3, E1 =  4, F1 =  5, G1 =  6, H1 =  7,
    A2 =  8, B2 =  9, C2 = 10, D2 = 11, E2 = 12, F2 = 13, G2 = 14, H2 = 15,
    A3 = 16, B3 = 17, C3 = 18, D3 = 19, E3 = 20, F3 = 21, G3 = 22, H3 = 23,
    A4 = 24, B4 = 25, C4 = 26, D4 = 27, E4 = 28, F4 = 29, G4 = 30, H4 = 31,
    A5 = 32, B5 = 33, C5 = 34, D5 = 35, E5 = 36, F5 = 37, G5 = 38, H5 = 39,
    A6 = 40, B6 = 41, C6 = 42, D6 = 43, E6 = 44, F6 = 45, G6 = 46, H6 = 47,
    A7 = 48, B7 = 49, C7 = 50, D7 = 51, E7 = 52, F7 = 53, G7 = 54, H7 = 55,
    A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63,
}

#[rustfmt::skip]
pub const ALL_SQUARES: [Square; Square::NUM] = [
    Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
    Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
    Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
    Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
    Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
    Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
    Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
    Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
];

impl FromStr for Square {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 2 {
            return Err(TypeError::InvalidSquare {
                square: s.to_string(),
            });
        }

        let file = match File::from_str(&s[0..1]) {
            Ok(file) => file,
            Err(_) => {
                return Err(TypeError::InvalidSquare {
                    square: s.to_string(),
                });
            }
        };

        let rank = match Rank::from_str(&s[1..2]) {
            Ok(rank) => rank,
            Err(_) => {
                return Err(TypeError::InvalidSquare {
                    square: s.to_string(),
                });
            }
        };

        Ok(Square::new(file, rank))
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

impl Square {
    /// Number of squares on a chessboard
    pub const NUM: usize = 64;

    /// Create a new Square from a File and Rank
    #[inline(always)]
    pub const fn new(file: File, rank: Rank) -> Self {
        let index = (rank as u8) * 8 + (file as u8);
        Self::from_index(index as i8)
    }

    /// Create a Square from an index (0-63)
    ///
    /// # Safety invariant
    /// This function uses unsafe transmute for performance in hot paths (called
    /// millions of times per second in search). The safety is guaranteed by:
    /// - Square is #[repr(u8)] with values 0-63 matching the index
    /// - All callers within this crate use valid indices (e.g., from bitboard iteration)
    /// - External callers should use `try_from_index()` for untrusted input
    /// - debug_assert validates bounds in debug builds
    #[inline(always)]
    pub const fn from_index(index: i8) -> Self {
        debug_assert!(index >= 0 && index < 64, "Square index out of range");
        // SAFETY: Square is repr(u8) with explicit values 0-63, callers ensure valid index
        unsafe { std::mem::transmute(index as u8) }
    }

    pub const fn try_from_index(index: i8) -> Option<Self> {
        if index < 0 || index >= 64 {
            None
        } else {
            Some(Self::from_index(index))
        }
    }

    /// Convert the Square to an index (0-63)
    #[inline(always)]
    pub const fn to_index(self) -> u8 {
        self as u8
    }

    /// Get the File of the Square
    #[inline(always)]
    pub const fn file(self) -> File {
        File::from_index(((self as u8) % 8) as i8)
    }

    /// Get the Rank of the Square
    #[inline(always)]
    pub const fn rank(self) -> Rank {
        Rank::from_index(((self as u8) / 8) as i8)
    }

    /// Get the BitBoard representation of the Square
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(1 << self as u8)
    }

    /// Offset the Square by the given file and rank deltas
    pub const fn offset(self, file: i8, rank: i8) -> Option<Self> {
        let file = self.file() as i8 + file;
        let rank = self.rank() as i8 + rank;
        if file < 0 || file >= 8 || rank < 0 || rank >= 8 {
            None
        } else {
            Some(Square::new(File::from_index(file), Rank::from_index(rank)))
        }
    }

    /// Flip the Square horizontally (mirror across vertical axis)
    pub const fn flip_file(self) -> Self {
        Self::new(self.file().flip(), self.rank())
    }

    /// Flip the Square vertically (mirror across horizontal axis)
    pub const fn flip_rank(self) -> Self {
        Self::new(self.file(), self.rank().flip())
    }

    /// Get the Square relative to the given color's perspective
    pub const fn relative_to(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => self.flip_rank(),
        }
    }

    /// Move the Square up by one rank relative to the given color
    pub const fn up(self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.offset(0, 1),
            Color::Black => self.offset(0, -1),
        }
    }

    /// Move the Square down by one rank relative to the given color
    pub const fn down(self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.offset(0, -1),
            Color::Black => self.offset(0, 1),
        }
    }

    /// Move the Square left by one file relative to the given color
    pub const fn left(self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.offset(-1, 0),
            Color::Black => self.offset(1, 0),
        }
    }

    /// Move the Square right by one file relative to the given color
    pub const fn right(self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.offset(1, 0),
            Color::Black => self.offset(-1, 0),
        }
    }
}
