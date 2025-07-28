use aether_types::{BitBoard, Color, Rank, Square};

pub static PAWN_ATTACKS: [[BitBoard; Square::NUM]; 2] = [
    init_pawn_attacks(Color::White),
    init_pawn_attacks(Color::Black),
];

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
    PAWN_ATTACKS[color as usize][sq.to_index() as usize]
}

pub fn get_pawn_attacks_to_square(sq: Square, color: Color) -> BitBoard {
    let mut attacks = BitBoard::EMPTY;

    match color {
        Color::White => {
            if let Some(target) = sq.offset(-1, 1) {
                attacks |= BitBoard::from_square(target);
            }
            if let Some(target) = sq.offset(1, 1) {
                attacks |= BitBoard::from_square(target);
            }
        }
        Color::Black => {
            if let Some(target) = sq.offset(-1, -1) {
                attacks |= BitBoard::from_square(target);
            }
            if let Some(target) = sq.offset(1, -1) {
                attacks |= BitBoard::from_square(target);
            }
        }
    }

    attacks
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

/// True if `sq` is the promotion rank for `color`.
#[inline(always)]
pub fn is_promotion_rank(sq: Square, color: Color) -> bool {
    match color {
        Color::White => sq.rank() == Rank::Eight,
        Color::Black => sq.rank() == Rank::One,
    }
}
