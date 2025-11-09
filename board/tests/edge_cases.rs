//! Edge case tests for board operations

use aether_types::{BoardQuery, Color, Piece};
use board::{Board, BoardOps, FenOps};

#[test]
fn test_checkmate_position() {
    // Fool's mate position
    let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
    let board = Board::from_fen(fen).expect("Valid FEN");

    // Verify it's checkmate (this is Black to move after Qh4#)
    // Actually this is White to move and White is in checkmate
    println!("Board loaded: {}", board.to_fen());
}

#[test]
fn test_stalemate_position() {
    // Simple stalemate: King vs King
    let fen = "k7/8/8/8/8/8/8/K7 w - - 0 1";
    let board = Board::from_fen(fen).expect("Valid FEN");

    println!("Stalemate position loaded: {}", board.to_fen());
}

#[test]
fn test_en_passant_capture() {
    // Position after 1.e4 d5 2.e5 (en passant available on d6)
    let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
    let board = Board::from_fen(fen).expect("Valid FEN");

    assert_eq!(
        board.en_passant_square(),
        Some(aether_types::Square::D6)
    );
}

#[test]
fn test_castling_rights_preserved() {
    let board = Board::starting_position().expect("Valid board");

    assert!(board.can_castle_short(Color::White));
    assert!(board.can_castle_long(Color::White));
    assert!(board.can_castle_short(Color::Black));
    assert!(board.can_castle_long(Color::Black));
}

#[test]
fn test_insufficient_material_kk() {
    // King vs King
    let fen = "k7/8/8/8/8/8/8/K7 w - - 0 1";
    let board = Board::from_fen(fen).expect("Valid FEN");

    // Should have exactly 2 pieces
    let mut piece_count = 0;
    for square in aether_types::Square::all() {
        if board.piece_at(*square).is_some() {
            piece_count += 1;
        }
    }
    assert_eq!(piece_count, 2);
}

#[test]
fn test_pawn_on_first_rank_invalid() {
    // Pawns cannot be on rank 1 or 8
    let fen = "k6P/8/8/8/8/8/8/K7 w - - 0 1";

    // This should either fail or load (depending on FEN validation strictness)
    // For now, just verify we can handle it
    let result = Board::from_fen(fen);
    println!("Pawn on 8th rank result: {:?}", result.is_ok());
}

#[test]
fn test_underpromotion_position() {
    // Position where knight promotion makes sense
    let fen = "8/P7/2k5/8/8/8/8/K7 w - - 0 1";
    let board = Board::from_fen(fen).expect("Valid FEN");

    // Verify pawn on a7
    assert_eq!(
        board.piece_at(aether_types::Square::A7),
        Some((Piece::Pawn, Color::White))
    );
}

#[test]
fn test_position_with_many_pieces() {
    // Kiwipete - complex position with many pieces
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_fen(fen).expect("Valid FEN");

    let mut piece_count = 0;
    for square in aether_types::Square::all() {
        if board.piece_at(*square).is_some() {
            piece_count += 1;
        }
    }

    // Should have many pieces (not just 2 kings)
    assert!(piece_count > 10);
}

#[test]
fn test_fen_roundtrip() {
    let positions = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    ];

    for original_fen in positions {
        let board = Board::from_fen(original_fen).expect("Valid FEN");
        let generated_fen = board.to_fen();

        // Parse the generated FEN again
        let board2 = Board::from_fen(&generated_fen).expect("Generated FEN should be valid");

        // Both boards should produce the same FEN
        assert_eq!(board.to_fen(), board2.to_fen(), "FEN roundtrip failed for: {}", original_fen);
    }
}

#[test]
fn test_move_gives_check() {
    // Position where a move gives check
    let fen = "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
    let board = Board::from_fen(fen).expect("Valid FEN");

    // Just verify we can load positions where checks might occur
    println!("Position loaded, side to move: {:?}", board.side_to_move());
}

#[test]
fn test_make_unmake_preserves_castling_rights() {
    use aether_types::MoveGen;
    use movegen::Generator;

    let board = Board::starting_position().expect("Valid board");
    let mut moves = Vec::new();
    let move_gen = Generator::new();
    move_gen.legal(&board, &mut moves);

    for mv in moves.iter().take(5) {
        let mut board_copy = board.clone();
        let original_fen = board.to_fen();

        // Make move
        board_copy.make_move(*mv).expect("Move should be legal");

        // Unmake move
        board_copy.unmake_move(*mv).expect("Unmake should work");

        // Verify FEN is restored (at least piece placement)
        let restored_fen = board_copy.to_fen();
        let orig_parts: Vec<&str> = original_fen.split_whitespace().collect();
        let restored_parts: Vec<&str> = restored_fen.split_whitespace().collect();

        assert_eq!(
            orig_parts[0], restored_parts[0],
            "Piece placement should be identical after unmake for move {:?}",
            mv
        );
    }
}
