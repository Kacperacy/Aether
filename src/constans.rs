use crate::bitboard::Bitboard;

pub const BOARD_SIZE: usize = 64;
pub const BOARD_WIDTH: usize = 8;
pub const MOVE_UP: i32 = 8;
pub const MOVE_DOWN: i32 = -8;
pub const MOVE_LEFT: i32 = -1;
pub const MOVE_RIGHT: i32 = 1;
pub const EMPTY: Bitboard = Bitboard(0x0000000000000000);
pub const ROW_1: Bitboard = Bitboard(0x00000000000000FF);
pub const ROW_2: Bitboard = Bitboard(0x000000000000FF00);
pub const ROW_3: Bitboard = Bitboard(0x0000000000FF0000);
pub const ROW_4: Bitboard = Bitboard(0x00000000FF000000);
pub const ROW_5: Bitboard = Bitboard(0x000000FF00000000);
pub const ROW_6: Bitboard = Bitboard(0x0000FF0000000000);
pub const ROW_7: Bitboard = Bitboard(0x00FF000000000000);
pub const ROW_8: Bitboard = Bitboard(0xFF00000000000000);

pub const COL_A: Bitboard = Bitboard(0x0101010101010101);
pub const COL_B: Bitboard = Bitboard(0x0202020202020202);
pub const COL_C: Bitboard = Bitboard(0x0404040404040404);
pub const COL_D: Bitboard = Bitboard(0x0808080808080808);
pub const COL_E: Bitboard = Bitboard(0x1010101010101010);
pub const COL_F: Bitboard = Bitboard(0x2020202020202020);
pub const COL_G: Bitboard = Bitboard(0x4040404040404040);
pub const COL_H: Bitboard = Bitboard(0x8080808080808080);

pub const STARTING_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub const ROOK_DIRECTIONS: [i32; 4] = [1, -1, 8, -8];
pub const BISHOP_DIRECTIONS: [i32; 4] = [9, 7, -7, -9];
pub const KNIGHT_DIRECTIONS: [i32; 8] = [6, 10, 15, 17, -6, -10, -15, -17];
pub const KING_DIRECTIONS: [i32; 8] = [1, 9, 8, 7, -1, -9, -8, -7];
pub const QUEEN_DIRECTIONS: [i32; 8] = [1, 9, 8, 7, -1, -9, -8, -7];
