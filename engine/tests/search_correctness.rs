//! Search integrity invariants.
//!
//! These assert properties the search must satisfy for *any* position, rather
//! than comparing against a table of expected best moves. That makes them
//! self-verifying: nothing here depends on hand-entered reference data that
//! could itself be wrong. Move generation is the trusted base (it is checked to
//! billions of nodes by the deep perft tier), so everything is validated against
//! it.

use aether_core::Move;
use board::Board;
use engine::Engine;
use engine::bench::BENCH_POSITIONS;
use engine::search::SearchLimits;

fn legal_moves(board: &Board) -> movegen::MoveList {
    let mut moves = movegen::MoveList::new();
    movegen::legal(board, &mut moves);
    moves
}

/// Whatever the search returns must be a move you are actually allowed to play.
#[test]
fn test_best_move_is_always_legal() {
    let mut engine = Engine::new(8);

    for fen in BENCH_POSITIONS {
        let mut board: Board = fen.parse().expect("valid FEN");
        let legal = legal_moves(&board);

        engine.new_game();
        let result = engine.search(&mut board, &SearchLimits::depth(6), |_, _, _| {});

        let Some(best) = result.best_move else {
            assert!(
                legal.is_empty(),
                "no best move returned but {fen} has moves"
            );
            continue;
        };

        assert!(
            legal.contains(&best),
            "search returned illegal move {best} in {fen}"
        );
    }
}

/// The whole principal variation must be playable, move by move, from the root.
///
/// This is the strongest cheap check on search integrity: a PV containing an
/// illegal move means some part of the tree searched a position it should never
/// have reached.
#[test]
fn test_principal_variation_is_playable() {
    let mut engine = Engine::new(8);

    for fen in BENCH_POSITIONS {
        let mut board: Board = fen.parse().expect("valid FEN");

        engine.new_game();
        let result = engine.search(&mut board, &SearchLimits::depth(6), |_, _, _| {});
        let pv: Vec<Move> = result.pv().to_vec();

        let mut replay: Board = fen.parse().expect("valid FEN");
        for (i, mv) in pv.iter().enumerate() {
            let legal = legal_moves(&replay);
            assert!(
                legal.contains(mv),
                "PV move {} ({mv}) at index {i} is illegal in {fen}\nPV: {pv:?}",
                i + 1
            );
            replay.make_move(mv).expect("legal move must apply");
        }
    }
}

/// A claimed mate must actually be mate.
///
/// Rather than trusting a table of "mate in N" positions, this plays the engine's
/// own PV out and checks the final position really is checkmate — no legal moves
/// and the side to move in check. A search that inflates mate scores fails here.
#[test]
fn test_claimed_mates_are_real() {
    // Simple forced mates; the assertion does not depend on these being mate in
    // any particular number of moves, only that a claimed mate is genuine.
    let positions = [
        "6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1",
        "7k/6pp/8/8/8/8/6PP/R6K w - - 0 1",
        "6k1/8/6K1/8/8/8/8/7Q w - - 0 1",
        "3k4/8/3K4/8/8/8/8/7R w - - 0 1",
    ];

    let mut engine = Engine::new(8);
    let mut mates_found = 0;

    for fen in positions {
        let mut board: Board = fen.parse().expect("valid FEN");
        engine.new_game();
        let result = engine.search(&mut board, &SearchLimits::depth(6), |_, _, _| {});

        let Some(mate_in) = engine::eval::score_to_mate_moves(result.score) else {
            continue;
        };
        if mate_in <= 0 {
            continue; // being mated, not delivering mate
        }
        mates_found += 1;

        // Play the PV out and verify the end position is genuinely checkmate.
        let mut replay: Board = fen.parse().expect("valid FEN");
        for mv in result.pv() {
            let legal = legal_moves(&replay);
            assert!(legal.contains(mv), "PV move {mv} illegal in {fen}");
            replay.make_move(mv).expect("legal move must apply");
        }

        assert!(
            legal_moves(&replay).is_empty(),
            "engine claimed mate in {mate_in} for {fen}, but the PV ends in a position \
             with legal moves"
        );
        assert!(
            replay.is_in_check(replay.side_to_move()),
            "engine claimed mate in {mate_in} for {fen}, but the PV ends in stalemate"
        );
    }

    assert!(
        mates_found > 0,
        "expected the engine to find at least one forced mate in this set"
    );
}

/// Searching the same position twice from a clean engine must give the same
/// answer. Catches accidental dependence on state that survives `new_game()`.
#[test]
fn test_search_is_reproducible() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut engine = Engine::new(8);

    let mut first_nodes = 0;
    let mut first_move = None;

    for run in 0..2 {
        let mut board: Board = fen.parse().expect("valid FEN");
        engine.new_game();
        let result = engine.search(&mut board, &SearchLimits::depth(7), |_, _, _| {});

        if run == 0 {
            first_nodes = result.info.nodes;
            first_move = result.best_move;
        } else {
            assert_eq!(
                result.info.nodes, first_nodes,
                "node count not reproducible"
            );
            assert_eq!(result.best_move, first_move, "best move not reproducible");
        }
    }
}

/// Limits must combine, not compete.
///
/// `create_search_limits` used to be an if/else cascade that picked exactly one
/// limit, so `go depth N` was silently discarded whenever a clock was also
/// present. Each limit now applies simultaneously.
#[test]
fn test_limits_apply_simultaneously() {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut engine = Engine::new(8);

    // A generous depth paired with a tight node cap: the node cap must bite.
    let limits = SearchLimits {
        depth: Some(64),
        nodes: Some(20_000),
        ..SearchLimits::default()
    };

    let mut board: Board = fen.parse().expect("valid FEN");
    engine.new_game();
    let result = engine.search(&mut board, &limits, |_, _, _| {});

    assert!(
        result.info.nodes < 200_000,
        "node limit ignored: searched {} nodes with a 20k cap",
        result.info.nodes
    );

    // And the reverse: a tight depth paired with a huge node cap must stop on depth.
    let limits = SearchLimits {
        depth: Some(4),
        nodes: Some(100_000_000),
        ..SearchLimits::default()
    };

    let mut board: Board = fen.parse().expect("valid FEN");
    engine.new_game();
    let result = engine.search(&mut board, &limits, |_, _, _| {});

    assert!(
        result.info.depth <= 4,
        "depth limit ignored: reached depth {}",
        result.info.depth
    );
}
