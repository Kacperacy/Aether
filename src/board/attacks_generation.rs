use crate::bitboard::Bitboard;
use crate::board::{Board, Color, Piece};
use crate::constants::*;

pub fn print_attacks(attacks: &Bitboard) {
    for i in 0..BOARD_SIZE {
        if i % BOARD_WIDTH == 0 {
            println!();
        }

        if attacks.is_set(i) {
            print!("X ");
        } else {
            print!(". ");
        }
    }

    println!();
}
// pub fn generate_pawn_attacks(&self) -> Bitboard {
//     let mut attacks = Bitboard::new();
//     let pawns = self.pieces[self.turn as usize][Piece::Pawn as usize];
//
//     let direction = match self.turn {
//         Color::White => MOVE_UP,
//         Color::Black => MOVE_DOWN,
//     };
//
//     for i in 0..BOARD_SIZE {
//         if !pawns.is_set(i) {
//             continue;
//         }
//
//         let from = i;
//         let left = from as i32 + direction + MOVE_LEFT;
//         let right = from as i32 + direction + MOVE_RIGHT;
//
//         if Board::is_index_in_bounds(left) {
//             if (left % BOARD_WIDTH as i32 - (from % BOARD_WIDTH) as i32).abs() > 1 {
//                 continue;
//             }
//             attacks.set_bit(left as usize);
//         }
//         if Board::is_index_in_bounds(right) {
//             if (right % BOARD_WIDTH as i32 - (from % BOARD_WIDTH) as i32).abs() > 1 {
//                 continue;
//             }
//             attacks.set_bit(right as usize);
//         }
//     }
//
//     attacks
// }

// pub fn generate_knight_attacks(&self) -> Bitboard {
//     let knights = self.pieces[self.turn as usize][Piece::Knight as usize];
//
//     let mut attacks = Bitboard::new();
//
//     for i in 0..BOARD_SIZE {
//         if !knights.is_set(i) {
//             continue;
//         }
//
//         let from = i;
//         for direction in KNIGHT_DIRECTIONS.iter() {
//             let to = from as i32 + direction;
//             if !Board::is_index_in_bounds(to) {
//                 continue;
//             }
//
//             if (to % BOARD_WIDTH as i32 - (from % BOARD_WIDTH) as i32).abs() > 2 {
//                 continue;
//             }
//
//             attacks.set_bit(to as usize);
//         }
//     }
//
//     attacks
// }

// pub fn generate_bishop_attacks(square: usize, all_occupancy: Bitboard) -> Bitboard {
// }
//
// pub fn generate_rook_attacks() -> Bitboard {
// }

// pub fn generate_queen_attacks(&self) -> Bitboard {
//     let queens = self.pieces[self.turn as usize][Piece::Queen as usize];
//
//     self.generate_slider_attacks(&QUEEN_DIRECTIONS, queens)
// }
//
// pub fn generate_king_attacks(&self) -> Bitboard {
//     let king = self.pieces[self.turn as usize][Piece::King as usize];
//
//     let mut attacks = Bitboard::new();
//
//     for i in 0..BOARD_SIZE {
//         if !king.is_set(i) {
//             continue;
//         }
//
//         let from = i;
//         for direction in KING_DIRECTIONS.iter() {
//             let to = from as i32 + direction;
//             if !Board::is_index_in_bounds(to) {
//                 continue;
//             }
//
//             if (to % BOARD_WIDTH as i32 - (from % BOARD_WIDTH) as i32).abs() > 1 {
//                 continue;
//             }
//
//             attacks.set_bit(to as usize);
//         }
//     }
//
//     attacks
// }
