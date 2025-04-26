use super::utils::generate_offsets_table;
use aether_types::{BitBoard, Square};

static KING_MOVES: [BitBoard; Square::NUM] = generate_offsets_table([
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
]);

pub fn get_king_moves(sq: Square) -> BitBoard {
    KING_MOVES[sq as usize]
}
