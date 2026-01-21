use aether_core::{
    BitBoard, Color, File, Move, MoveFlags, PROMOTION_PIECES, Piece, Square, bishop_attacks,
    is_promotion_rank, king_attacks, knight_attacks, pawn_attacks, pawn_moves, queen_attacks,
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
fn push_move(
    moves: &mut Vec<Move>,
    from: Square,
    to: Square,
    piece: Piece,
    capture: Option<Piece>,
    flags: MoveFlags,
    promotion: Option<Piece>,
) {
    let mut chess_move = Move::new(from, to, piece).with_flags(flags);

    if let Some(captured_piece) = capture {
        chess_move = chess_move.with_capture(captured_piece);
    }

    if let Some(promotion_piece) = promotion {
        chess_move = chess_move.with_promotion(promotion_piece);
    }

    moves.push(chess_move);
}

#[inline]
fn generate_piece_moves(
    board: &Board,
    from: Square,
    piece: Piece,
    targets: BitBoard,
    occupied: BitBoard,
    moves: &mut Vec<Move>,
) {
    let flags = MoveFlags::default();

    for to in targets.iter() {
        let capture = if occupied.has(to) {
            board.piece_at(to).map(|(p, _)| p)
        } else {
            None
        };
        push_move(moves, from, to, piece, capture, flags, None);
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
    let normal_flags = MoveFlags::default();
    let double_push_flags = MoveFlags {
        is_double_pawn_push: true,
        ..MoveFlags::default()
    };

    let push_targets = pawn_moves(from, side, occupied);
    for to in push_targets.iter() {
        let is_promotion = is_promotion_rank(to, side);
        let is_double_push = to.rank().to_index().abs_diff(from.rank().to_index()) == 2;
        let flags = if is_double_push {
            double_push_flags
        } else {
            normal_flags
        };

        if is_promotion {
            for &promo_piece in &PROMOTION_PIECES {
                push_move(moves, from, to, Piece::Pawn, None, flags, Some(promo_piece));
            }
        } else {
            push_move(moves, from, to, Piece::Pawn, None, flags, None);
        }
    }

    let capture_targets = pawn_attacks(from, side) & opponent_pieces;
    for to in capture_targets.iter() {
        let captured = board.piece_at(to).map(|(p, _)| p);
        let is_promotion = is_promotion_rank(to, side);

        if is_promotion {
            for &promo_piece in &PROMOTION_PIECES {
                push_move(
                    moves,
                    from,
                    to,
                    Piece::Pawn,
                    captured,
                    normal_flags,
                    Some(promo_piece),
                );
            }
        } else if captured.is_some() {
            push_move(moves, from, to, Piece::Pawn, captured, normal_flags, None);
        }
    }

    if let Some(ep_square) = board.en_passant_square() {
        if pawn_attacks(from, side).has(ep_square) {
            let ep_flags = MoveFlags {
                is_en_passant: true,
                ..MoveFlags::default()
            };
            push_move(
                moves,
                from,
                ep_square,
                Piece::Pawn,
                Some(Piece::Pawn),
                ep_flags,
                None,
            );
        }
    }
}

fn generate_knight_moves(
    board: &Board,
    from: Square,
    occupied: BitBoard,
    own_pieces: BitBoard,
    moves: &mut Vec<Move>,
) {
    let targets = knight_attacks(from) & !own_pieces;
    generate_piece_moves(board, from, Piece::Knight, targets, occupied, moves);
}

fn generate_slider_moves(
    board: &Board,
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
    generate_piece_moves(board, from, piece, targets, occupied, moves);
}

fn generate_king_moves(
    board: &Board,
    from: Square,
    occupied: BitBoard,
    own_pieces: BitBoard,
    moves: &mut Vec<Move>,
) {
    let targets = king_attacks(from) & !own_pieces;
    generate_piece_moves(board, from, Piece::King, targets, occupied, moves);

    if let Some((_, side)) = board.piece_at(from) {
        generate_castling_moves(board, from, side, moves);
    }
}

fn generate_castling_moves(board: &Board, king_square: Square, side: Color, moves: &mut Vec<Move>) {
    let opponent = side.opponent();
    let castle_flags = MoveFlags {
        is_castle: true,
        ..MoveFlags::default()
    };

    if board.can_castle_short(side) {
        let back = side.back_rank();
        let king_start = Square::new(File::E, back);
        let f_square = Square::new(File::F, back);
        let g_square = Square::new(File::G, back);

        let path_clear = !board.is_square_occupied(f_square) && !board.is_square_occupied(g_square);
        let path_safe = !board.is_square_attacked(king_start, opponent)
            && !board.is_square_attacked(f_square, opponent)
            && !board.is_square_attacked(g_square, opponent);

        if king_square == king_start && path_clear && path_safe {
            push_move(
                moves,
                king_start,
                g_square,
                Piece::King,
                None,
                castle_flags,
                None,
            );
        }
    }

    if board.can_castle_long(side) {
        let back = side.back_rank();
        let king_start = Square::new(File::E, back);
        let d_square = Square::new(File::D, back);
        let c_square = Square::new(File::C, back);
        let b_square = Square::new(File::B, back);

        let path_clear = !board.is_square_occupied(d_square)
            && !board.is_square_occupied(c_square)
            && !board.is_square_occupied(b_square);
        let path_safe = !board.is_square_attacked(king_start, opponent)
            && !board.is_square_attacked(d_square, opponent)
            && !board.is_square_attacked(c_square, opponent);

        if king_square == king_start && path_clear && path_safe {
            push_move(
                moves,
                king_start,
                c_square,
                Piece::King,
                None,
                castle_flags,
                None,
            );
        }
    }
}

pub fn pseudo_legal(board: &Board, moves: &mut Vec<Move>) {
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
                Piece::Knight => generate_knight_moves(board, square, occupied, own_pieces, moves),
                Piece::Bishop | Piece::Rook | Piece::Queen => {
                    generate_slider_moves(board, square, piece, occupied, own_pieces, moves)
                }
                Piece::King => generate_king_moves(board, square, occupied, own_pieces, moves),
            }
        }
    }
}

pub fn legal(board: &Board, moves: &mut Vec<Move>) {
    pseudo_legal(board, moves);
    moves.retain(|mv| !board.would_leave_king_in_check(mv));
}

pub fn captures(board: &Board, moves: &mut Vec<Move>) {
    pseudo_legal(board, moves);
    moves.retain(|m| m.is_capture() || m.flags.is_en_passant);
}

pub fn quiet_moves(board: &Board, moves: &mut Vec<Move>) {
    pseudo_legal(board, moves);
    moves.retain(|m| !m.is_capture() && !m.flags.is_en_passant && !m.flags.is_castle);
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

    let flags = MoveFlags::default();

    let knights = board.piece_bb(Piece::Knight, side);
    for from in knights.iter() {
        let targets = knight_attacks(from) & knight_check_sqs & !all_occ;
        for to in targets.iter() {
            push_move(moves, from, to, Piece::Knight, None, flags, None);
        }
    }

    let bishops = board.piece_bb(Piece::Bishop, side);
    for from in bishops.iter() {
        let targets = bishop_attacks(from, all_occ) & bishop_check_sqs & !all_occ;
        for to in targets.iter() {
            push_move(moves, from, to, Piece::Bishop, None, flags, None);
        }
    }

    let rooks = board.piece_bb(Piece::Rook, side);
    for from in rooks.iter() {
        let targets = rook_attacks(from, all_occ) & rook_check_sqs & !all_occ;
        for to in targets.iter() {
            push_move(moves, from, to, Piece::Rook, None, flags, None);
        }
    }

    let queens = board.piece_bb(Piece::Queen, side);
    let queen_check_sqs = bishop_check_sqs | rook_check_sqs;
    for from in queens.iter() {
        let targets = queen_attacks(from, all_occ) & queen_check_sqs & !all_occ;
        for to in targets.iter() {
            push_move(moves, from, to, Piece::Queen, None, flags, None);
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
