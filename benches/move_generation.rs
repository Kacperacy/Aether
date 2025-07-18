// use board::Board;
// use criterion::{Criterion, criterion_group, criterion_main};
// use movegen::MoveGenerator;
// use std::hint::black_box;
//
// fn benchmark_move_generation(c: &mut Criterion) {
//     let board = Board::starting_position().expect("Failed to create board");
//     let generator = MoveGenerator::new();
//
//     c.bench_function("generate_all_moves", |b| {
//         b.iter(|| {
//             let moves = generator.generate_all_moves(black_box(&board));
//             black_box(moves)
//         })
//     });
//
//     c.bench_function("generate_legal_moves", |b| {
//         b.iter(|| {
//             let moves = generator.generate_legal_moves(black_box(&board));
//             black_box(moves)
//         })
//     });
// }
//
// fn benchmark_perft(c: &mut Criterion) {
//     let board = Board::starting_position().expect("Failed to create board");
//     let generator = MoveGenerator::new();
//
//     c.bench_function("perft_4", |b| {
//         b.iter(|| {
//             let nodes = generator.debug_perft(black_box(&board), 4);
//             black_box(nodes)
//         })
//     });
// }
//
// criterion_group!(benches, benchmark_move_generation, benchmark_perft);
// criterion_main!(benches);
