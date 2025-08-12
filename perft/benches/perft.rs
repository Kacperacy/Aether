use board::{Board, FenOps};
use criterion::{Criterion, criterion_group, criterion_main};
use perft::perft_count;
use std::hint::black_box;

fn bench_startpos_d3(c: &mut Criterion) {
    let board = Board::starting_position().expect("failed to build starting position");
    c.bench_function("perft startpos d3", |b| {
        b.iter(|| black_box(perft_count(&board, 3)))
    });
}

fn bench_kiwipete_d2(c: &mut Criterion) {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_fen(fen).expect("failed to parse kiwipete");
    c.bench_function("perft kiwipete d2", |b| {
        b.iter(|| black_box(perft_count(&board, 2)))
    });
}

fn bench_pos3_d4(c: &mut Criterion) {
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";
    let board = Board::from_fen(fen).expect("failed to parse pos3");
    c.bench_function("perft pos3 d4", |b| {
        b.iter(|| black_box(perft_count(&board, 4)))
    });
}

fn benches(c: &mut Criterion) {
    bench_startpos_d3(c);
    bench_kiwipete_d2(c);
    bench_pos3_d4(c);
}

criterion_group!(perft_benches, benches);
criterion_main!(perft_benches);
