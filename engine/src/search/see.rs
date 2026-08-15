//! Static exchange evaluation.
//!
//! Works directly against the board so it can use the O(1) mailbox lookup and
//! the cached per-color occupancy rather than rebuilding them from bitboards.

use crate::eval::{Score, material};
use aether_core::{BitBoard, Color, Move, Piece, Square};
use attacks::{bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks};
use board::Board;

#[inline]
pub fn see_value(board: &Board, mv: &Move) -> Score {
    let side = board.side_to_move();
    let pieces = board.pieces();
    let to = mv.to_sq();
    let from = mv.from_sq();

    let target_value = if mv.is_en_passant() {
        material::PAWN_VALUE
    } else if mv.is_capture() {
        match board.piece_at(to) {
            Some((piece, c)) if c != side => material::value(piece),
            _ => return 0,
        }
    } else {
        return 0;
    };

    let (moving_piece, _) = board.piece_at(from).expect("SEE: no piece at from-square");

    let (promotion_gain, attacker_value) = match mv.promotion_piece() {
        Some(promo_piece) => {
            let promo_value = material::value(promo_piece);
            (promo_value - material::PAWN_VALUE, promo_value)
        }
        None => (0, material::value(moving_piece)),
    };

    let mut gain: [Score; 32] = [0; 32];
    let mut depth = 0;

    gain[0] = target_value + promotion_gain;

    let mut occ = board.occupied() ^ from.bitboard() ^ to.bitboard();

    let mut attackers = all_attackers_to_square(to, occ, pieces);

    if matches!(moving_piece, Piece::Pawn | Piece::Bishop | Piece::Queen) {
        attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
    }
    if matches!(moving_piece, Piece::Rook | Piece::Queen) {
        attackers |= rook_attacks(to, occ) & get_straight_sliders(pieces);
    }
    attackers &= occ;

    let mut current_piece_value = attacker_value;
    let mut stm = side.opponent();

    while let Some((attacker_sq, attacker_piece)) =
        get_least_valuable_attacker(attackers & board.occupied_by(stm), pieces, stm)
    {
        depth += 1;

        gain[depth] = current_piece_value - gain[depth - 1];

        if (-gain[depth - 1]).max(gain[depth]) < 0 {
            break;
        }

        occ ^= attacker_sq.bitboard();

        if matches!(attacker_piece, Piece::Pawn | Piece::Bishop | Piece::Queen) {
            attackers |= bishop_attacks(to, occ) & get_diagonal_sliders(pieces);
        }
        if matches!(attacker_piece, Piece::Rook | Piece::Queen) {
            attackers |= rook_attacks(to, occ) & get_straight_sliders(pieces);
        }

        attackers &= occ;
        current_piece_value = material::value(attacker_piece);
        stm = stm.opponent();
    }

    while depth > 0 {
        gain[depth - 1] = -(-gain[depth - 1]).max(gain[depth]);
        depth -= 1;
    }

    gain[0]
}

#[inline]
fn all_attackers_to_square(
    square: Square,
    occupied: BitBoard,
    pieces: &[[BitBoard; 6]; 2],
) -> BitBoard {
    let white = &pieces[Color::White as usize];
    let black = &pieces[Color::Black as usize];

    let white_pawn_attackers = pawn_attacks(square, Color::Black) & white[Piece::Pawn as usize];
    let black_pawn_attackers = pawn_attacks(square, Color::White) & black[Piece::Pawn as usize];

    let knight_attackers =
        knight_attacks(square) & (white[Piece::Knight as usize] | black[Piece::Knight as usize]);

    let diagonal_attackers = bishop_attacks(square, occupied) & get_diagonal_sliders(pieces);
    let straight_attackers = rook_attacks(square, occupied) & get_straight_sliders(pieces);

    let king_attackers =
        king_attacks(square) & (white[Piece::King as usize] | black[Piece::King as usize]);

    white_pawn_attackers
        | black_pawn_attackers
        | knight_attackers
        | diagonal_attackers
        | straight_attackers
        | king_attackers
}

#[inline]
fn get_diagonal_sliders(pieces: &[[BitBoard; 6]; 2]) -> BitBoard {
    pieces[Color::White as usize][Piece::Bishop as usize]
        | pieces[Color::White as usize][Piece::Queen as usize]
        | pieces[Color::Black as usize][Piece::Bishop as usize]
        | pieces[Color::Black as usize][Piece::Queen as usize]
}

#[inline]
fn get_straight_sliders(pieces: &[[BitBoard; 6]; 2]) -> BitBoard {
    pieces[Color::White as usize][Piece::Rook as usize]
        | pieces[Color::White as usize][Piece::Queen as usize]
        | pieces[Color::Black as usize][Piece::Rook as usize]
        | pieces[Color::Black as usize][Piece::Queen as usize]
}

#[inline]
fn get_least_valuable_attacker(
    attackers: BitBoard,
    pieces: &[[BitBoard; 6]; 2],
    color: Color,
) -> Option<(Square, Piece)> {
    const PIECE_ORDER: [Piece; 6] = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    let color_pieces = &pieces[color as usize];
    for &piece in &PIECE_ORDER {
        let piece_attackers = attackers & color_pieces[piece as usize];
        if !piece_attackers.is_empty() {
            return Some((piece_attackers.lsb(), piece));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::material::{BISHOP_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE};

    use crate::search::test_support::pos;

    fn capture(from: Square, to: Square) -> Move {
        Move::new(from, to, Move::CAPTURE)
    }

    #[test]
    fn test_simple_pawn_takes_pawn() {
        let b = pos("7k/8/8/3p4/4P3/8/8/K7 w - - 0 1");
        let mv = capture(Square::E4, Square::D5);

        assert_eq!(see_value(&b, &mv), PAWN_VALUE);
    }

    #[test]
    fn test_pawn_takes_defended_pawn() {
        let b = pos("7k/8/2p5/3p4/4P3/8/8/K7 w - - 0 1");
        let mv = capture(Square::E4, Square::D5);

        assert_eq!(see_value(&b, &mv), 0);
    }

    #[test]
    fn test_queen_takes_defended_pawn_loses() {
        let b = pos("7k/8/4p3/3p4/8/8/8/K2Q4 w - - 0 1");
        let mv = capture(Square::D1, Square::D5);

        assert_eq!(see_value(&b, &mv), PAWN_VALUE - QUEEN_VALUE);
    }

    #[test]
    fn test_knight_takes_defended_rook() {
        let b = pos("7k/8/5p2/4r3/8/5N2/8/K7 w - - 0 1");
        let mv = capture(Square::F3, Square::E5);
        let expected = ROOK_VALUE - KNIGHT_VALUE;

        assert_eq!(see_value(&b, &mv), expected);
    }

    #[test]
    fn test_xray_rook_battery_wins_queen() {
        // Rooks doubled on the d-file: the rear rook re-captures.
        let b = pos("3q3k/8/8/8/3R4/8/8/K2R4 w - - 0 1");
        let mv = capture(Square::D4, Square::D8);

        assert_eq!(see_value(&b, &mv), QUEEN_VALUE);
    }

    #[test]
    fn test_undefended_bishop_capture() {
        let b = pos("7k/8/8/8/R2Rb3/8/8/K7 w - - 0 1");
        let mv = capture(Square::D4, Square::E4);

        assert_eq!(see_value(&b, &mv), BISHOP_VALUE);
    }

    #[test]
    fn test_knight_takes_bishop_defended_by_queen() {
        let b = pos("7k/4q3/8/8/4b3/2N5/8/K7 w - - 0 1");
        let mv = capture(Square::C3, Square::E4);

        // NxB then QxN nets the bishop-knight difference.
        assert_eq!(see_value(&b, &mv), BISHOP_VALUE - KNIGHT_VALUE);
    }

    #[test]
    fn test_equal_exchange() {
        let b = pos("7k/8/8/8/4n3/2N5/8/K7 w - - 0 1");
        let mv = capture(Square::C3, Square::E4);

        assert_eq!(see_value(&b, &mv), KNIGHT_VALUE);
    }

    #[test]
    fn test_defended_equal_exchange() {
        let b = pos("7k/8/8/6n1/4n3/2N5/8/K7 w - - 0 1");
        let mv = capture(Square::C3, Square::E4);

        assert_eq!(see_value(&b, &mv), 0);
    }

    #[test]
    fn test_quiet_move_has_no_exchange_value() {
        let b = pos("7k/8/8/8/8/2N5/8/K7 w - - 0 1");
        let quiet = Move::new(Square::C3, Square::E4, Move::QUIET);

        assert_eq!(see_value(&b, &quiet), 0);
    }
}
