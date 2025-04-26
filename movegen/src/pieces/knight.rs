use super::utils::generate_offsets_table;
use aether_types::{BitBoard, Square};

static KNIGHT_MOVES: [BitBoard; Square::NUM] = generate_offsets_table([
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
]);

pub fn get_knight_moves(sq: Square) -> BitBoard {
    KNIGHT_MOVES[sq as usize]
}
