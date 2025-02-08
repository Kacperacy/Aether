use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl FromStr for Piece {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "p" => Ok(Self::Pawn),
            "n" => Ok(Self::Knight),
            "b" => Ok(Self::Bishop),
            "r" => Ok(Self::Rook),
            "q" => Ok(Self::Queen),
            "k" => Ok(Self::King),
            _ => Err(()),
        }
    }
}
