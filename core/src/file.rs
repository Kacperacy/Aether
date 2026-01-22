use crate::CoreError::InvalidFile;
use crate::{BitBoard, CoreError, Result};
use std::fmt::Display;
use std::str::FromStr;

pub const FILE_MASKS: [u64; 8] = [
    0x0101010101010101,
    0x0202020202020202,
    0x0404040404040404,
    0x0808080808080808,
    0x1010101010101010,
    0x2020202020202020,
    0x4040404040404040,
    0x8080808080808080,
];

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
    type Err = CoreError;

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
    pub const NUM: usize = 8;

    #[inline(always)]
    pub const fn from_index(file: i8) -> Self {
        debug_assert!(file >= 0 && file < 8, "File index out of bounds");
        // SAFETY: File is #[repr(u8)] with variants 0..=7, and we assert file is in 0..8.
        unsafe { std::mem::transmute(file as u8) }
    }

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

    pub const fn offset(self, offset: i8) -> Option<Self> {
        let new_file = self as i8 + offset;
        if new_file < 0 || new_file > 7 {
            None
        } else {
            Some(Self::from_index(new_file))
        }
    }

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

    #[inline(always)]
    pub const fn to_index(self) -> u8 {
        self as u8
    }
}
