use crate::{Piece, Square};
use std::fmt::{self, Display, Formatter};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub piece: Piece,
    pub capture: Option<Piece>,
    pub promotion: Option<Piece>,
    pub flags: MoveFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveFlags {
    pub is_castle: bool,
    pub is_en_passant: bool,
    pub is_double_pawn_push: bool,
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // 1. origin + destination squares
        write!(f, "{}{}", self.from, self.to)?;

        // 2. optional promotion suffix (UCI uses lowercase piece letters)
        if let Some(p) = self.promotion {
            let symbol = match p {
                Piece::Knight => 'n',
                Piece::Bishop => 'b',
                Piece::Rook => 'r',
                Piece::Queen => 'q',
                // King and Pawn can never appear here – unreachable by rule
                _ => unreachable!("illegal promotion piece"),
            };
            write!(f, "{symbol}")?;
        }

        Ok(())
    }
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Move {
            from,
            to,
            piece: Piece::Pawn, // This should be properly determined
            capture: None,
            promotion: None,
            flags: MoveFlags {
                is_castle: false,
                is_en_passant: false,
                is_double_pawn_push: false,
            },
        }
    }

    pub fn with_promotion(mut self, piece: Piece) -> Self {
        self.promotion = Some(piece);
        self
    }

    pub fn is_capture(&self) -> bool {
        self.capture.is_some()
    }
}
