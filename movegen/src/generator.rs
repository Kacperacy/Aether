use aether_core::{BitBoard, CastlingPath, Color, Move, Piece, Square};
use attacks::{
    bishop_attacks, king_attacks, knight_attacks, pawn_attacks, pawn_moves, queen_attacks,
    rook_attacks,
};
use board::Board;

#[inline]
fn occupancies(board: &Board, side: Color) -> (BitBoard, BitBoard, BitBoard) {
    let own = board.occupied_by(side);
    let opponent = board.occupied_by(side.opponent());
    let all = own | opponent;
    (all, own, opponent)
}

#[inline(always)]
fn promo_flag(piece: Piece, is_capture: bool) -> u16 {
    let idx = piece.to_index() as u16 - 1; // Knight=1..Queen=4 -> 0..3
    Move::PROMO_N + idx + if is_capture { 4 } else { 0 }
}

#[inline]
fn generate_piece_moves(
    from: Square,
    targets: BitBoard,
    occupied: BitBoard,
    moves: &mut Vec<Move>,
) {
    for to in targets.iter() {
        let flags = if occupied.contains(to) {
            Move::CAPTURE
        } else {
            Move::QUIET
        };
        moves.push(Move::new(from, to, flags));
    }
}

fn generate_pawn_moves(
    board: &Board,
    from: Square,
    side: Color,
    occupied: BitBoard,
    opponent_pieces: BitBoard,
    moves: &mut Vec<Move>,
) {
    let push_targets = pawn_moves(from, side, occupied);
    for to in push_targets.iter() {
        let is_promotion = to.is_promotion_rank(side);
        let is_double_push = to.rank().to_index().abs_diff(from.rank().to_index()) == 2;

        if is_promotion {
            for &promo_piece in &Piece::PROMOTIONS {
                moves.push(Move::new(from, to, promo_flag(promo_piece, false)));
            }
        } else {
            let flags = if is_double_push {
                Move::DOUBLE_PUSH
            } else {
                Move::QUIET
            };
            moves.push(Move::new(from, to, flags));
        }
    }

    let capture_targets = pawn_attacks(from, side) & opponent_pieces;
    for to in capture_targets.iter() {
        let is_promotion = to.is_promotion_rank(side);

        if is_promotion {
            for &promo_piece in &Piece::PROMOTIONS {
                moves.push(Move::new(from, to, promo_flag(promo_piece, true)));
            }
        } else {
            moves.push(Move::new(from, to, Move::CAPTURE));
        }
    }

    if let Some(ep_square) = board.en_passant_square()
        && pawn_attacks(from, side).contains(ep_square)
    {
        moves.push(Move::new(from, ep_square, Move::EN_PASSANT));
    }
}

fn generate_knight_moves(
    from: Square,
    occupied: BitBoard,
    own_pieces: BitBoard,
    moves: &mut Vec<Move>,
) {
    let targets = knight_attacks(from) & !own_pieces;
    generate_piece_moves(from, targets, occupied, moves);
}

fn generate_slider_moves(
    from: Square,
    piece: Piece,
    occupied: BitBoard,
    own_pieces: BitBoard,
    moves: &mut Vec<Move>,
) {
    let attacks = match piece {
        Piece::Bishop => bishop_attacks(from, occupied),
        Piece::Rook => rook_attacks(from, occupied),
        Piece::Queen => queen_attacks(from, occupied),
        _ => return,
    };
    let targets = attacks & !own_pieces;
    generate_piece_moves(from, targets, occupied, moves);
}

fn generate_king_moves(
    board: &Board,
    from: Square,
    occupied: BitBoard,
    own_pieces: BitBoard,
    moves: &mut Vec<Move>,
) {
    let targets = king_attacks(from) & !own_pieces;
    generate_piece_moves(from, targets, occupied, moves);

    if let Some((_, side)) = board.piece_at(from) {
        generate_castling_moves(board, from, side, moves);
    }
}

fn generate_castling_moves(board: &Board, king_square: Square, side: Color, moves: &mut Vec<Move>) {
    if board.can_castle_short(side) {
        try_castle(
            board,
            king_square,
            side,
            CastlingPath::kingside(side),
            Move::CASTLE_KS,
            moves,
        );
    }

    if board.can_castle_long(side) {
        try_castle(
            board,
            king_square,
            side,
            CastlingPath::queenside(side),
            Move::CASTLE_QS,
            moves,
        );
    }
}

#[inline]
fn try_castle(
    board: &Board,
    king_square: Square,
    side: Color,
    path: CastlingPath,
    flag: u16,
    moves: &mut Vec<Move>,
) {
    if king_square != path.king_from {
        return;
    }

    let opponent = side.opponent();
    let path_clear = path.vacancy.iter().all(|&sq| !board.is_square_occupied(sq));
    let path_safe = path
        .king_safety
        .iter()
        .all(|&sq| !board.is_square_attacked(sq, opponent));

    if path_clear && path_safe {
        moves.push(Move::new(path.king_from, path.king_to, flag));
    }
}

pub(crate) fn pseudo_legal(board: &Board, moves: &mut Vec<Move>) {
    moves.clear();
    moves.reserve(256);

    let side = board.side_to_move();
    let (occupied, own_pieces, opponent_pieces) = occupancies(board, side);

    for square in own_pieces.iter() {
        if let Some((piece, _)) = board.piece_at(square) {
            match piece {
                Piece::Pawn => {
                    generate_pawn_moves(board, square, side, occupied, opponent_pieces, moves)
                }
                Piece::Knight => generate_knight_moves(square, occupied, own_pieces, moves),
                Piece::Bishop | Piece::Rook | Piece::Queen => {
                    generate_slider_moves(square, piece, occupied, own_pieces, moves)
                }
                Piece::King => generate_king_moves(board, square, occupied, own_pieces, moves),
            }
        }
    }
}

pub fn legal(board: &Board, moves: &mut Vec<Move>) {
    pseudo_legal(board, moves);
    moves.retain(|mv| !crate::legality::would_leave_king_in_check(board, mv));
}

pub fn captures(board: &Board, moves: &mut Vec<Move>) {
    pseudo_legal(board, moves);
    moves.retain(|m| m.is_capture() || m.is_en_passant());
}

pub fn checks(board: &Board, moves: &mut Vec<Move>) {
    let side = board.side_to_move();
    let opponent = side.opponent();
    let king_sq = board.get_king_square(opponent);

    let own_pieces = board.occupied_by(side);
    let opp_pieces = board.occupied_by(opponent);
    let all_occ = own_pieces | opp_pieces;

    let knight_check_sqs = knight_attacks(king_sq);
    let bishop_check_sqs = bishop_attacks(king_sq, all_occ);
    let rook_check_sqs = rook_attacks(king_sq, all_occ);

    let knights = board.piece_bb(Piece::Knight, side);
    for from in knights.iter() {
        let targets = knight_attacks(from) & knight_check_sqs & !all_occ;
        for to in targets.iter() {
            moves.push(Move::new(from, to, Move::QUIET));
        }
    }

    let bishops = board.piece_bb(Piece::Bishop, side);
    for from in bishops.iter() {
        let targets = bishop_attacks(from, all_occ) & bishop_check_sqs & !all_occ;
        for to in targets.iter() {
            moves.push(Move::new(from, to, Move::QUIET));
        }
    }

    let rooks = board.piece_bb(Piece::Rook, side);
    for from in rooks.iter() {
        let targets = rook_attacks(from, all_occ) & rook_check_sqs & !all_occ;
        for to in targets.iter() {
            moves.push(Move::new(from, to, Move::QUIET));
        }
    }

    let queens = board.piece_bb(Piece::Queen, side);
    let queen_check_sqs = bishop_check_sqs | rook_check_sqs;
    for from in queens.iter() {
        let targets = queen_attacks(from, all_occ) & queen_check_sqs & !all_occ;
        for to in targets.iter() {
            moves.push(Move::new(from, to, Move::QUIET));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legal_moves_starting_position() {
        let board = Board::starting_position().unwrap();
        let mut moves = Vec::new();

        legal(&board, &mut moves);

        assert_eq!(
            moves.len(),
            20,
            "Starting position should have 20 legal moves"
        );
    }
}
