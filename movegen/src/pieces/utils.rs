use aether_types::{BitBoard, Square};

pub const fn generate_offsets_table<const N: usize>(
    offsets: [(i8, i8); N],
) -> [BitBoard; Square::NUM] {
    let mut moves = [BitBoard::EMPTY; Square::NUM];
    let mut i = 0;
    while i < Square::NUM {
        let sq = Square::from_index(i as i8);
        moves[i] = compute_moves_from_offsets(sq, &offsets);
        i += 1;
    }
    moves
}

pub const fn compute_moves_from_offsets<const N: usize>(
    sq: Square,
    offsets: &[(i8, i8); N],
) -> BitBoard {
    let mut result = BitBoard::EMPTY;
    let mut i = 0;
    while i < N {
        let (file_offset, rank_offset) = offsets[i];
        if let Some(target) = sq.offset(file_offset, rank_offset) {
            result = BitBoard(result.0 | target.bitboard().0);
        }
        i += 1;
    }
    result
}
