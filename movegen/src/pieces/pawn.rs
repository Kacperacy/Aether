use aether_types::{BitBoard, Color, Rank, Square};

static WHITE_PAWN_ATTACKS: [BitBoard; Square::NUM] = init_pawn_attacks(Color::White);
static BLACK_PAWN_ATTACKS: [BitBoard; Square::NUM] = init_pawn_attacks(Color::Black);

const fn init_pawn_attacks(color: Color) -> [BitBoard; Square::NUM] {
    let mut attacks = [BitBoard::EMPTY; Square::NUM];
    let mut i = 0;
    while i < Square::NUM {
        let sq = Square::from_index(i as i8);
        attacks[i] = compute_pawn_attacks(sq, color);
        i += 1;
    }
    attacks
}

const fn compute_pawn_attacks(sq: Square, color: Color) -> BitBoard {
    let mut result = BitBoard::EMPTY;

    match color {
        Color::White => {
            if let Some(target) = sq.offset(-1, 1) {
                result = BitBoard(result.0 | target.bitboard().0);
            }
            if let Some(target) = sq.offset(1, 1) {
                result = BitBoard(result.0 | target.bitboard().0);
            }
        }
        Color::Black => {
            if let Some(target) = sq.offset(-1, -1) {
                result = BitBoard(result.0 | target.bitboard().0);
            }
            if let Some(target) = sq.offset(1, -1) {
                result = BitBoard(result.0 | target.bitboard().0);
            }
        }
    }

    result
}

pub fn get_pawn_attacks(sq: Square, color: Color) -> BitBoard {
    match color {
        Color::White => WHITE_PAWN_ATTACKS[sq as usize],
        Color::Black => BLACK_PAWN_ATTACKS[sq as usize],
    }
}

pub fn get_pawn_moves(sq: Square, color: Color, occupied: BitBoard) -> BitBoard {
    let mut moves = BitBoard::EMPTY;

    // Single push
    let push_offset = if color == Color::White { 1 } else { -1 };
    if let Some(single_push) = sq.offset(0, push_offset) {
        if !occupied.has(single_push) {
            moves |= BitBoard::from_square(single_push);

            // Double push from starting rank
            let starting_rank = if color == Color::White {
                Rank::Two
            } else {
                Rank::Seven
            };
            if sq.rank() == starting_rank {
                if let Some(double_push) = sq.offset(0, push_offset * 2) {
                    if !occupied.has(double_push) {
                        moves |= BitBoard::from_square(double_push);
                    }
                }
            }
        }
    }

    moves
}
