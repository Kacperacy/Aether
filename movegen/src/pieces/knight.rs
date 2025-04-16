use aether_types::{BitBoard, Square};

static KNIGHT_MOVES: [BitBoard; Square::NUM] = init_knight_attacks();

const fn init_knight_attacks() -> [BitBoard; Square::NUM] {
    let mut attacks = [BitBoard::EMPTY; Square::NUM];
    let mut i = 0;
    while i < Square::NUM {
        let sq = Square::from_index(i as i8);
        attacks[i] = compute_knight_attacks(sq);
        i += 1;
    }
    attacks
}

const fn compute_knight_attacks(sq: Square) -> BitBoard {
    let mut result = BitBoard::EMPTY;
    let offsets = [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    let mut i = 0;
    while i < 8 {
        let (file_offset, rank_offset) = offsets[i];
        if let Some(target) = sq.offset(file_offset, rank_offset) {
            result = BitBoard(result.0 | target.bitboard().0);
        }
        i += 1;
    }
    result
}

pub fn get_knight_moves(sq: Square) -> BitBoard {
    KNIGHT_MOVES[sq as usize]
}
