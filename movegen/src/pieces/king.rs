use aether_types::{BitBoard, Square};

static KING_MOVES: [BitBoard; Square::NUM] = init_king_attacks();

const fn init_king_attacks() -> [BitBoard; Square::NUM] {
    let mut attacks = [BitBoard::EMPTY; Square::NUM];
    let mut i = 0;
    while i < Square::NUM {
        let sq = Square::from_index(i as i8);
        attacks[i] = compute_king_attacks(sq);
        i += 1;
    }
    attacks
}

const fn compute_king_attacks(sq: Square) -> BitBoard {
    let mut result = BitBoard::EMPTY;
    let offsets = [
        (0, 1),
        (1, 1),
        (1, 0),
        (1, -1),
        (0, -1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
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

pub fn get_king_moves(sq: Square) -> BitBoard {
    KING_MOVES[sq as usize]
}
