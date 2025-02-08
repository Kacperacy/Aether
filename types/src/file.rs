use crate::BitBoard;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl FromStr for File {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "a" => Ok(Self::A),
            "b" => Ok(Self::B),
            "c" => Ok(Self::C),
            "d" => Ok(Self::D),
            "e" => Ok(Self::E),
            "f" => Ok(Self::F),
            "g" => Ok(Self::G),
            "h" => Ok(Self::H),
            _ => Err(()),
        }
    }
}

impl File {
    pub const fn from_index(file: i8) -> Self {
        match file {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            6 => Self::G,
            7 => Self::H,
            _ => panic!("Invalid file"),
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
}
