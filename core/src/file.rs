use crate::TypeError::{InvalidFile, InvalidFileIndex};
use crate::{BitBoard, Result, TypeError};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

/// All files on a chessboard
pub const ALL_FILES: [File; 8] = [
    File::A,
    File::B,
    File::C,
    File::D,
    File::E,
    File::F,
    File::G,
    File::H,
];

impl FromStr for File {
    type Err = TypeError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            "c" => Ok(Self::C),
            "d" => Ok(Self::D),
            "e" => Ok(Self::E),
            "f" => Ok(Self::F),
            "g" => Ok(Self::G),
            "h" => Ok(Self::H),
            _ => Err(InvalidFile {
                file: s.to_string(),
            }),
        }
    }
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl File {
    /// Number of files on a chessboard
    pub const NUM: usize = 8;

    /// Safe conversion from index (0-7) to File
    pub fn try_from_index(file: u8) -> Result<Self> {
        match file {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::C),
            3 => Ok(Self::D),
            4 => Ok(Self::E),
            5 => Ok(Self::F),
            6 => Ok(Self::G),
            7 => Ok(Self::H),
            _ => Err(InvalidFileIndex { file_index: file }),
        }
    }

    /// Unsafe conversion from index (0-7) to File
    #[inline(always)]
    pub const fn from_index(file: i8) -> Self {
        debug_assert!(file >= 0 && file < 8, "File index out of bounds");
        // SAFETY:
        // - File is repr(u8) with explicit values 0-7
        // - debug_assert checks bounds in debug mode
        unsafe { std::mem::transmute(file as u8) }
    }

    /// Returns the character representation of the File
    pub const fn as_char(self) -> char {
        match self {
            Self::A => 'a',
            Self::B => 'b',
            Self::C => 'c',
            Self::D => 'd',
            Self::E => 'e',
            Self::F => 'f',
            Self::G => 'g',
            Self::H => 'h',
        }
    }

    /// Returns a new File offset by the given amount, or None if out of bounds
    pub const fn offset(self, offset: i8) -> Option<Self> {
        let new_file = self as i8 + offset;
        if new_file < 0 || new_file > 7 {
            None
        } else {
            Some(Self::from_index(new_file))
        }
    }

    /// Returns the File flipped horizontally (A<->H, B<->G, etc.)
    pub const fn flip(self) -> Self {
        match self {
            Self::A => Self::H,
            Self::B => Self::G,
            Self::C => Self::F,
            Self::D => Self::E,
            Self::E => Self::D,
            Self::F => Self::C,
            Self::G => Self::B,
            Self::H => Self::A,
        }
    }

    /// Returns the BitBoard representation of the File
    pub const fn bitboard(self) -> BitBoard {
        match self {
            Self::A => BitBoard(0x0101010101010101),
            Self::B => BitBoard(0x0202020202020202),
            Self::C => BitBoard(0x0404040404040404),
            Self::D => BitBoard(0x0808080808080808),
            Self::E => BitBoard(0x1010101010101010),
            Self::F => BitBoard(0x2020202020202020),
            Self::G => BitBoard(0x4040404040404040),
            Self::H => BitBoard(0x8080808080808080),
        }
    }

    /// Returns the BitBoard of squares adjacent to this File
    pub const fn adjacent(self) -> BitBoard {
        match self {
            Self::A => BitBoard(0x202020202020202),
            Self::B => BitBoard(0x505050505050505),
            Self::C => BitBoard(0xa0a0a0a0a0a0a0a),
            Self::D => BitBoard(0x1414141414141414),
            Self::E => BitBoard(0x2828282828282828),
            Self::F => BitBoard(0x5050505050505050),
            Self::G => BitBoard(0xa0a0a0a0a0a0a0a0),
            Self::H => BitBoard(0x4040404040404040),
        }
    }

    /// Converts the File to its corresponding index (0-7)
    #[inline(always)]
    pub const fn to_index(self) -> u8 {
        self as u8
    }
}
