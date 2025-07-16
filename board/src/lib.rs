mod check;
mod fen;
mod movement;
mod zobrist;

use aether_types::{BitBoard, CastlingRights, Color, File, Piece, Rank, Square};

#[derive(Clone, Debug)]
pub struct Board {
    /// 12 individual piece bitboards: [color][piece_type]
    /// White = 0, Black = 1
    /// Pawn = 0, Knight = 1, Bishop = 2, Rook = 3, Queen = 4, King = 5
    pieces: [[BitBoard; 6]; 2],

    /// Aggregate bitboards for optimization
    color_combined: [BitBoard; 2], // All white pieces, all black pieces
    occupied: BitBoard, // All pieces combined

    /// Game state information
    side_to_move: Color,
    castling_rights: [CastlingRights; 2],
    en_passant_square: Option<Square>,
    halfmove_clock: u8,
    fullmove_number: u16,

    /// Hash for transposition table
    zobrist_hash: u64,

    /// Cached check status
    /// 0 = White in check, 1 = Black in check
    cached_check_status: [Option<bool>; 2],
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            pieces: [[BitBoard::default(); 6]; 2],
            color_combined: [BitBoard::default(); 2],
            occupied: BitBoard::default(),
            side_to_move: Color::White,
            castling_rights: [CastlingRights::EMPTY; 2],
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            zobrist_hash: 0,
            cached_check_status: [None; 2],
        }
    }

    pub fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    pub fn en_passant(&self) -> Option<Square> {
        self.en_passant_square
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

    pub fn update_aggregate_bitboards(&mut self) {
        self.color_combined[Color::White as usize] = self.pieces[Color::White as usize]
            .iter()
            .fold(BitBoard::default(), |acc, &bb| acc | bb);
        self.color_combined[Color::Black as usize] = self.pieces[Color::Black as usize]
            .iter()
            .fold(BitBoard::default(), |acc, &bb| acc | bb);
        self.occupied =
            self.color_combined[Color::White as usize] | self.color_combined[Color::Black as usize];
    }

    pub fn get_king_square(&self, color: Color) -> Option<Square> {
        self.pieces[color as usize][Piece::King as usize].to_square()
    }

    pub fn invalidate_cached_check_status(&mut self) {
        self.cached_check_status[Color::White as usize] = None;
        self.cached_check_status[Color::Black as usize] = None;
    }

    pub fn get_cached_check_status(&mut self, color: Color) -> bool {
        if let Some(status) = self.cached_check_status[color as usize] {
            return status;
        }

        // If not cached, calculate and cache it
        let in_check = self.is_in_check(color);

        self.cached_check_status[color as usize] = Some(in_check);

        in_check
    }
}
