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
        match (file, rank) {
            (File::A, Rank::One) => Self::A1, (File::B, Rank::One) => Self::B1, (File::C, Rank::One) => Self::C1, (File::D, Rank::One) => Self::D1,
            (File::E, Rank::One) => Self::E1, (File::F, Rank::One) => Self::F1, (File::G, Rank::One) => Self::G1, (File::H, Rank::One) => Self::H1,
            (File::A, Rank::Two) => Self::A2, (File::B, Rank::Two) => Self::B2, (File::C, Rank::Two) => Self::C2, (File::D, Rank::Two) => Self::D2,
            (File::E, Rank::Two) => Self::E2, (File::F, Rank::Two) => Self::F2, (File::G, Rank::Two) => Self::G2, (File::H, Rank::Two) => Self::H2,
            (File::A, Rank::Three) => Self::A3, (File::B, Rank::Three) => Self::B3, (File::C, Rank::Three) => Self::C3, (File::D, Rank::Three) => Self::D3,
            (File::E, Rank::Three) => Self::E3, (File::F, Rank::Three) => Self::F3, (File::G, Rank::Three) => Self::G3, (File::H, Rank::Three) => Self::H3,
            (File::A, Rank::Four) => Self::A4, (File::B, Rank::Four) => Self::B4, (File::C, Rank::Four) => Self::C4, (File::D, Rank::Four) => Self::D4,
            (File::E, Rank::Four) => Self::E4, (File::F, Rank::Four) => Self::F4, (File::G, Rank::Four) => Self::G4, (File::H, Rank::Four) => Self::H4,
            (File::A, Rank::Five) => Self::A5, (File::B, Rank::Five) => Self::B5, (File::C, Rank::Five) => Self::C5, (File::D, Rank::Five) => Self::D5,
            (File::E, Rank::Five) => Self::E5, (File::F, Rank::Five) => Self::F5, (File::G, Rank::Five) => Self::G5, (File::H, Rank::Five) => Self::H5,
            (File::A, Rank::Six) => Self::A6, (File::B, Rank::Six) => Self::B6, (File::C, Rank::Six) => Self::C6, (File::D, Rank::Six) => Self::D6,
            (File::E, Rank::Six) => Self::E6, (File::F, Rank::Six) => Self::F6, (File::G, Rank::Six) => Self::G6, (File::H, Rank::Six) => Self::H6,
            (File::A, Rank::Seven) => Self::A7, (File::B, Rank::Seven) => Self::B7, (File::C, Rank::Seven) => Self::C7, (File::D, Rank::Seven) => Self::D7,
            (File::E, Rank::Seven) => Self::E7, (File::F, Rank::Seven) => Self::F7, (File::G, Rank::Seven) => Self::G7, (File::H, Rank::Seven) => Self::H7,
            (File::A, Rank::Eight) => Self::A8, (File::B, Rank::Eight) => Self::B8, (File::C, Rank::Eight) => Self::C8, (File::D, Rank::Eight) => Self::D8,
            (File::E, Rank::Eight) => Self::E8, (File::F, Rank::Eight) => Self::F8, (File::G, Rank::Eight) => Self::G8, (File::H, Rank::Eight) => Self::H8,
        }
    }

    pub const fn from_index(index: i8) -> Self {
        let file = File::from_index(index % 8);
        let rank = Rank::new(index / 8);
        Self::new(file, rank)
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
        if file < 0 || file <= 8 || rank < 0 || rank >= 8 {
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
