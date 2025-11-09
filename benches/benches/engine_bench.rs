//! Comprehensive benchmarks for the Aether chess engine.
//!
//! This benchmark suite measures performance of critical operations:
//! - Board operations (make_move, unmake_move)
//! - FEN parsing
//! - Move generation
//! - Position evaluation
//! - Transposition table operations
//! - Move ordering
//! - Search performance

use aether_types::{BoardQuery, Color, MoveGen, Piece, Square};
use board::{Board, BoardOps, FenOps};
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use eval::{Evaluator, SimpleEvaluator};
use movegen::Generator;
use search::{AlphaBetaSearcher, SearchLimits, Searcher, TranspositionTable};

// ============================================================================
// Board Operations Benchmarks
// ============================================================================

fn bench_make_unmake_move(c: &mut Criterion) {
    let mut group = c.benchmark_group("board_operations");

    let board = Board::starting_position().unwrap();
    let move_gen = Generator::new();
    let mut moves = Vec::new();
    move_gen.legal(&board, &mut moves);

    if let Some(first_move) = moves.first() {
        group.bench_function("make_move", |b| {
            b.iter(|| {
                let mut board_clone = black_box(board.clone());
                black_box(board_clone.make_move(*first_move).unwrap());
            })
        });

        group.bench_function("make_unmake_move", |b| {
            b.iter(|| {
                let mut board_clone = black_box(board.clone());
                board_clone.make_move(*first_move).unwrap();
                black_box(board_clone.unmake_move(*first_move).unwrap());
            })
        });
    }

    group.finish();
}

fn bench_fen_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("fen_operations");

    let fens = [
        ("startpos", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        ("kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
        ("complex", "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"),
    ];

    for (name, fen) in fens {
        group.bench_function(BenchmarkId::new("parse_fen", name), |b| {
            b.iter(|| black_box(Board::from_fen(fen).unwrap()))
        });

        let board = Board::from_fen(fen).unwrap();
        group.bench_function(BenchmarkId::new("to_fen", name), |b| {
            b.iter(|| black_box(board.to_fen()))
        });
    }

    group.finish();
}

// ============================================================================
// Move Generation Benchmarks
// ============================================================================

fn bench_move_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_generation");

    let positions = [
        ("startpos", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        ("kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
        ("endgame", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
        ("middlegame", "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4"),
    ];

    let move_gen = Generator::new();

    for (name, fen) in positions {
        let board = Board::from_fen(fen).unwrap();
        group.bench_function(BenchmarkId::new("generate_all_moves", name), |b| {
            b.iter(|| black_box(move_gen.generate_moves(&board)))
        });
    }

    group.finish();
}

// ============================================================================
// Evaluation Benchmarks
// ============================================================================

fn bench_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation");

    let positions = [
        ("startpos", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        ("kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
        ("endgame", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
        ("tactical", "r1bqkb1r/pppp1ppp/2n2n2/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4"),
    ];

    let evaluator = SimpleEvaluator::new();

    for (name, fen) in positions {
        let board = Board::from_fen(fen).unwrap();
        group.bench_function(BenchmarkId::new("evaluate", name), |b| {
            b.iter(|| black_box(evaluator.evaluate(&board)))
        });
    }

    group.finish();
}

// ============================================================================
// Transposition Table Benchmarks
// ============================================================================

fn bench_transposition_table(c: &mut Criterion) {
    let mut group = c.benchmark_group("transposition_table");

    // Benchmark TT operations
    let mut tt = TranspositionTable::new(16); // 16 MB
    let test_hash = 0x123456789ABCDEF0u64;

    group.bench_function("tt_store", |b| {
        b.iter(|| {
            black_box(tt.store(
                test_hash,
                None,
                100,
                5,
                search::EntryType::Exact,
            ))
        })
    });

    tt.store(test_hash, None, 100, 5, search::EntryType::Exact);

    group.bench_function("tt_probe_hit", |b| {
        b.iter(|| black_box(tt.probe(test_hash)))
    });

    group.bench_function("tt_probe_miss", |b| {
        b.iter(|| black_box(tt.probe(test_hash + 1)))
    });

    group.finish();
}

// ============================================================================
// Search Benchmarks
// ============================================================================

fn bench_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("search");
    group.sample_size(10); // Reduce sample size for expensive benchmarks

    let positions = [
        ("startpos", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        ("tactical", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"),
    ];

    // Benchmark different search depths
    for depth in [3, 4, 5] {
        for (name, fen) in positions {
            let board = Board::from_fen(fen).unwrap();
            let mut searcher = AlphaBetaSearcher::new();
            let limits = SearchLimits::depth(depth);

            group.bench_function(
                BenchmarkId::new(format!("depth_{}", depth), name),
                |b| {
                    b.iter(|| {
                        black_box(searcher.search(&board, &limits))
                    })
                },
            );
        }
    }

    group.finish();
}

// ============================================================================
// Move Ordering Benchmarks
// ============================================================================

fn bench_move_ordering(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_ordering");

    let board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    let move_gen = Generator::new();
    let mut moves = Vec::new();
    move_gen.legal(&board, &mut moves);

    group.bench_function("simple_move_ordering", |b| {
        b.iter(|| {
            let mut moves_clone = black_box(moves.clone());
            let orderer = search::SimpleMoveOrderer::new();
            black_box(orderer.order_moves(&mut moves_clone));
        })
    });

    group.bench_function("advanced_move_ordering", |b| {
        b.iter(|| {
            let mut moves_clone = black_box(moves.clone());
            let orderer = search::AdvancedMoveOrderer::new();
            black_box(orderer.order_moves(&mut moves_clone));
        })
    });

    group.finish();
}

// ============================================================================
// Board Query Benchmarks
// ============================================================================

fn bench_board_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("board_query");

    let board = Board::starting_position().unwrap();
    let test_square = Square::from_index(8); // a2

    group.bench_function("piece_at", |b| {
        b.iter(|| black_box(board.piece_at(test_square)))
    });

    group.bench_function("is_square_occupied", |b| {
        b.iter(|| black_box(board.is_square_occupied(test_square)))
    });

    group.bench_function("is_square_attacked", |b| {
        b.iter(|| black_box(board.is_square_attacked(test_square, Color::Black)))
    });

    group.bench_function("get_king_square", |b| {
        b.iter(|| black_box(board.get_king_square(Color::White)))
    });

    group.finish();
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    board_benches,
    bench_make_unmake_move,
    bench_fen_operations,
    bench_board_query,
);

criterion_group!(
    movegen_benches,
    bench_move_generation,
);

criterion_group!(
    eval_benches,
    bench_evaluation,
);

criterion_group!(
    search_benches,
    bench_transposition_table,
    bench_move_ordering,
    bench_search,
);

criterion_main!(
    board_benches,
    movegen_benches,
    eval_benches,
    search_benches,
);
