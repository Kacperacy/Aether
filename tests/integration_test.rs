//! Integration tests for the Aether chess engine.
//!
//! These tests verify that all components work together correctly
//! in realistic scenarios.

use aether_types::{BoardQuery, MoveGen};
use board::{Board, BoardOps, FenOps};
use eval::{Evaluator, SimpleEvaluator};
use movegen::Generator;
use search::{AlphaBetaSearcher, SearchLimits, Searcher};

/// Helper function to generate legal moves
fn generate_legal_moves<T: BoardQuery>(board: &T) -> Vec<aether_types::Move> {
    let mut moves = Vec::new();
    let generator = Generator::new();
    generator.legal(board, &mut moves);
    moves
}

#[test]
fn test_complete_game_workflow() {
    // Create a new board
    let mut board = Board::starting_position().expect("Failed to create starting position");
    let evaluator = SimpleEvaluator::new();

    // Play a few moves
    let moves_to_play = [
        ("e2e4", "e2-e4 pawn move"),
        ("e7e5", "e7-e5 pawn move"),
        ("g1f3", "Nf3 knight move"),
        ("b8c6", "Nc6 knight move"),
    ];

    for (move_str, description) in &moves_to_play {
        // Generate legal moves
        let legal_moves = generate_legal_moves(&board);
        assert!(!legal_moves.is_empty(), "No legal moves at {}", description);

        // Find the move we want to play
        let mv = legal_moves
            .iter()
            .find(|m| {
                let from_str = format!("{}", m.from);
                let to_str = format!("{}", m.to);
                format!("{}{}", from_str.to_lowercase(), to_str.to_lowercase()) == *move_str
            })
            .unwrap_or_else(|| panic!("Move {} not found in legal moves", move_str));

        // Make the move
        board.make_move(*mv).unwrap_or_else(|_| panic!("Failed to make move {}", move_str));

        // Evaluate the position
        let score = evaluator.evaluate(&board);
        println!("After {}: score = {}", description, score);
    }

    // Verify we can still generate moves
    let final_moves = generate_legal_moves(&board);
    assert!(!final_moves.is_empty(), "No legal moves in final position");
}

#[test]
fn test_search_tactical_position() {
    // Kiwipete position - complex tactical position
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_fen(fen).expect("Failed to parse FEN");

    let mut searcher = AlphaBetaSearcher::new();
    let limits = SearchLimits::depth(4);

    let result = searcher.search(&board, &limits);

    assert!(result.best_move.is_some(), "Search should find a move");
    assert!(!result.pv.is_empty(), "Search should produce a principal variation");

    println!("Best move: {:?}", result.best_move.unwrap());
    println!("Score: {}", result.score);
    println!("Nodes: {}", result.info.nodes);
    println!("NPS: {}", result.info.nps);
}

#[test]
fn test_make_unmake_preserves_position() {
    let board = Board::starting_position().expect("Failed to create starting position");
    let moves = generate_legal_moves(&board);

    // Test make/unmake for all legal moves
    for mv in moves {
        let mut board_copy = board.clone();
        let original_fen = board.to_fen();

        // Make move
        board_copy.make_move(mv).expect("Failed to make move");

        // Unmake move
        board_copy.unmake_move(mv).expect("Failed to unmake move");

        // Verify position is restored
        let restored_fen = board_copy.to_fen();

        // Compare just the piece placement (first part of FEN)
        let original_placement: Vec<&str> = original_fen.split_whitespace().collect();
        let restored_placement: Vec<&str> = restored_fen.split_whitespace().collect();

        assert_eq!(
            original_placement[0], restored_placement[0],
            "Piece placement should be restored after make/unmake for move {:?}",
            mv
        );
    }
}

#[test]
fn test_search_improves_with_depth() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_fen(fen).expect("Failed to parse FEN");

    let mut nodes_per_depth = Vec::new();

    for depth in 1..=4 {
        let mut searcher = AlphaBetaSearcher::new();
        let limits = SearchLimits::depth(depth);
        let result = searcher.search(&board, &limits);

        nodes_per_depth.push((depth, result.info.nodes));
        println!("Depth {}: {} nodes", depth, result.info.nodes);
    }

    // Verify nodes increase with depth
    for i in 1..nodes_per_depth.len() {
        assert!(
            nodes_per_depth[i].1 > nodes_per_depth[i - 1].1,
            "Nodes should increase with depth: depth {} had {} nodes, depth {} had {} nodes",
            nodes_per_depth[i - 1].0,
            nodes_per_depth[i - 1].1,
            nodes_per_depth[i].0,
            nodes_per_depth[i].1
        );
    }
}

#[test]
fn test_evaluation_symmetry() {
    // Starting position should evaluate to roughly equal
    let board = Board::starting_position().expect("Failed to create starting position");
    let evaluator = SimpleEvaluator::new();

    let score = evaluator.evaluate(&board);

    // Score from White's perspective should be close to 0 (equal position)
    assert!(
        score.abs() < 100,
        "Starting position should evaluate close to 0, got {}",
        score
    );
}

#[test]
fn test_move_generation_count() {
    // Starting position should have 20 legal moves
    let board = Board::starting_position().expect("Failed to create starting position");
    let moves = generate_legal_moves(&board);

    assert_eq!(moves.len(), 20, "Starting position should have 20 legal moves");
}

#[test]
fn test_pawn_promotion() {
    // Position where White has a pawn ready to promote
    let fen = "8/P7/8/8/8/8/8/K6k w - - 0 1";
    let mut board = Board::from_fen(fen).expect("Failed to parse FEN");
    let moves = generate_legal_moves(&board);

    // Find promotion move
    let promotion_move = moves
        .iter()
        .find(|m| m.promotion.is_some())
        .expect("Should find a promotion move");

    println!("Promotion move: {:?}", promotion_move);

    // Make the promotion
    board.make_move(*promotion_move).expect("Failed to make promotion move");

    // Verify a piece was placed on the promotion square
    let promoted_square = promotion_move.to;
    let piece = board.piece_at(promoted_square);

    assert!(piece.is_some(), "Should have a piece on the promotion square");
    println!("Promoted piece: {:?}", piece.unwrap());
}

#[test]
fn test_en_passant_available() {
    // Position where en passant is possible
    let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1";
    let board = Board::from_fen(fen).expect("Failed to parse FEN");
    let moves = generate_legal_moves(&board);

    // Find en passant move
    let ep_move = moves
        .iter()
        .find(|m| m.flags.is_en_passant)
        .expect("Should find an en passant move");

    println!("En passant move: {:?}", ep_move);

    // Verify the move is correct (e5xd6)
    assert_eq!(ep_move.piece, aether_types::Piece::Pawn);
}

#[test]
fn test_transposition_table_effectiveness() {
    let board = Board::starting_position().expect("Failed to create starting position");

    let mut searcher = AlphaBetaSearcher::new();
    let limits = SearchLimits::depth(5);

    let result = searcher.search(&board, &limits);

    // Get TT statistics
    println!("Search completed:");
    println!("  Nodes: {}", result.info.nodes);
    println!("  Hash full: {}", result.info.hash_full);

    // TT should have some hits in a depth 5 search
    assert!(result.info.hash_full > 0, "Transposition table should have some entries");
}
