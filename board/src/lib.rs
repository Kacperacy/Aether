mod fen;
mod movement;

use aether_types::{BitBoard, CastlingRights, Color, File, Rank, Square};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Board {
    pieces: [BitBoard; 6],
    colors: [BitBoard; 2],
    side_to_move: Color,
    castling_rights: [CastlingRights; 2],
    en_passant: Option<Square>,
    halfmove_clock: u8,
    fullmove_number: u16,
}

impl Board {
    pub fn new() -> Self {
        Self {
            pieces: [BitBoard::EMPTY; 6],
            colors: [BitBoard::EMPTY; 2],
            side_to_move: Color::White,
            castling_rights: [CastlingRights::EMPTY; 2],
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    pub fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    pub fn fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    pub fn can_castle_short(&self, color: Color) -> bool {
        self.castling_rights[color as usize].short.is_some()
    }

    pub fn can_castle_long(&self, color: Color) -> bool {
        self.castling_rights[color as usize].long.is_some()
    }

    pub fn print(&self) {
        println!("  +-----------------+");
        for rank in (Rank::One as i8..=Rank::Eight as i8).rev() {
            print!("{} | ", rank + 1); // Rank indicator (1-8)
            for file in File::A as i8..=File::H as i8 {
                let square = Square::new(File::from_index(file), Rank::new(rank));
                if let Some((piece, color)) = self.piece_at(square) {
                    if color == Color::White {
                        print!("{} ", piece.as_char().to_ascii_uppercase());
                    } else {
                        print!("{} ", piece.as_char());
                    }
                } else {
                    print!(". ");
                }
            }
            println!("|");
        }
        println!("  +-----------------+");
        println!("    A B C D E F G H");
    }
}
