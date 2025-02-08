use crate::{Piece, Square};
use std::str::FromStr;

pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Piece>,
}

impl FromStr for Move {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let from = Square::from_str(&s[0..2]).map_err(|_| ())?;
        let to = Square::from_str(&s[2..4]).map_err(|_| ())?;
        let promotion = if let Some(promotion) = s.get(4..5) {
            let piece = Piece::from_str(promotion).map_err(|_| ())?;
            if piece == Piece::Queen
                || piece == Piece::Rook
                || piece == Piece::Bishop
                || piece == Piece::Knight
            {
                Some(piece)
            } else {
                return Err(());
            }
        } else {
            None
        };

        Ok(Self {
            from,
            to,
            promotion,
        })
    }
}
