use crate::{BitBoard, Color, File, Rank};
use std::str::FromStr;

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

#[rustfmt::skip]
pub const ALL_SQUARES: [Square; 64] = [
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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let file = chars.next().ok_or(())?;
        let rank = chars.next().ok_or(())?;
        if chars.next().is_some() {
            return Err(());
        }
        Ok(Square::new(
            File::from_str(&file.to_string())?,
            Rank::from_str(&rank.to_string())?,
        ))
    }
}

impl Square {
    #[rustfmt::skip]
    pub const fn new(file: File, rank: Rank) -> Self {
        let index = rank as i8 * 8 + file as i8;
        ALL_SQUARES[index as usize]
    }

    pub const fn from_index(index: i8) -> Self {
        let file = File::from_index(index % 8);
        let rank = Rank::new(index / 8);
        Self::new(file, rank)
    }

    pub const fn to_index(self) -> u8 {
        (self as u8) % 64
    }

    pub fn from_algebraic(algebraic: &str) -> Result<Self, &'static str> {
        if algebraic.len() != 2 {
            return Err("Algebraic notation must be exactly 2 characters");
        }

        let file = match File::from_str(&algebraic[0..1]) {
            Ok(file) => file,
            Err(_) => return Err("Invalid file character (must be a-h)"),
        };

        let rank = match Rank::from_str(&algebraic[1..2]) {
            Ok(rank) => rank,
            Err(_) => return Err("Invalid rank character (must be 1-8)"),
        };

        Ok(Self::new(file, rank))
    }

    pub fn to_algebraic(self) -> String {
        format!("{}{}", self.file().as_char(), self.rank() as i8)
    }

    #[rustfmt::skip]
    pub const fn file(self) -> File {
        match self {
            Self::A1 | Self::A2 | Self::A3 | Self::A4 | Self::A5 | Self::A6 | Self::A7 | Self::A8 => File::A,
            Self::B1 | Self::B2 | Self::B3 | Self::B4 | Self::B5 | Self::B6 | Self::B7 | Self::B8 => File::B,
            Self::C1 | Self::C2 | Self::C3 | Self::C4 | Self::C5 | Self::C6 | Self::C7 | Self::C8 => File::C,
            Self::D1 | Self::D2 | Self::D3 | Self::D4 | Self::D5 | Self::D6 | Self::D7 | Self::D8 => File::D,
            Self::E1 | Self::E2 | Self::E3 | Self::E4 | Self::E5 | Self::E6 | Self::E7 | Self::E8 => File::E,
            Self::F1 | Self::F2 | Self::F3 | Self::F4 | Self::F5 | Self::F6 | Self::F7 | Self::F8 => File::F,
            Self::G1 | Self::G2 | Self::G3 | Self::G4 | Self::G5 | Self::G6 | Self::G7 | Self::G8 => File::G,
            Self::H1 | Self::H2 | Self::H3 | Self::H4 | Self::H5 | Self::H6 | Self::H7 | Self::H8 => File::H,
        }
    }

    #[rustfmt::skip]
    pub const fn rank(self) -> Rank {
        match self {
            Self::A1 | Self::B1 | Self::C1 | Self::D1 | Self::E1 | Self::F1 | Self::G1 | Self::H1 => Rank::One,
            Self::A2 | Self::B2 | Self::C2 | Self::D2 | Self::E2 | Self::F2 | Self::G2 | Self::H2 => Rank::Two,
            Self::A3 | Self::B3 | Self::C3 | Self::D3 | Self::E3 | Self::F3 | Self::G3 | Self::H3 => Rank::Three,
            Self::A4 | Self::B4 | Self::C4 | Self::D4 | Self::E4 | Self::F4 | Self::G4 | Self::H4 => Rank::Four,
            Self::A5 | Self::B5 | Self::C5 | Self::D5 | Self::E5 | Self::F5 | Self::G5 | Self::H5 => Rank::Five,
            Self::A6 | Self::B6 | Self::C6 | Self::D6 | Self::E6 | Self::F6 | Self::G6 | Self::H6 => Rank::Six,
            Self::A7 | Self::B7 | Self::C7 | Self::D7 | Self::E7 | Self::F7 | Self::G7 | Self::H7 => Rank::Seven,
            Self::A8 | Self::B8 | Self::C8 | Self::D8 | Self::E8 | Self::F8 | Self::G8 | Self::H8 => Rank::Eight,
        }
    }

    pub const fn bitboard(self) -> BitBoard {
        BitBoard(1 << self as u8)
    }

    pub const fn offset(self, file: i8, rank: i8) -> Option<Self> {
        let file = self.file() as i8 + file;
        let rank = self.rank() as i8 + rank;
        if file < 0 || file >= 8 || rank < 0 || rank >= 8 {
            None
        } else {
            Some(Square::new(File::from_index(file), Rank::new(rank)))
        }
    }

    pub const fn flip_file(self) -> Self {
        Self::new(self.file().flip(), self.rank())
    }

    pub const fn flip_rank(self) -> Self {
        Self::new(self.file(), self.rank().flip())
    }

    pub const fn relative_to(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => self.flip_rank(),
        }
    }

    pub const fn up(self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.offset(0, 1),
            Color::Black => self.offset(0, -1),
        }
    }

    pub const fn down(self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.offset(0, -1),
            Color::Black => self.offset(0, 1),
        }
    }

    pub const fn left(self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.offset(-1, 0),
            Color::Black => self.offset(1, 0),
        }
    }

    pub const fn right(self, color: Color) -> Option<Self> {
        match color {
            Color::White => self.offset(1, 0),
            Color::Black => self.offset(-1, 0),
        }
    }
}
