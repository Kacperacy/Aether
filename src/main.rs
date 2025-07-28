use aether_types::MoveGen;
use board::{Board, FenOps, STARTING_POSITION_FEN};
use movegen::{Generator, magic_gen};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let constants_path = "movegen/src/magic_constants.rs";

    if args.len() > 1 && args[1] == "generate-magics" {
        println!("Generating magic bitboard constants...");

        if let Err(e) = magic_gen::generate_magic_constants(constants_path) {
            eprintln!("Failed to generate magic constants: {e}");
            std::process::exit(1);
        }

        println!("Magic constants generation complete!");
    }

    let mut board = Board::from_fen(STARTING_POSITION_FEN).unwrap();
    let generator = Generator::default();
    let mut moves = Vec::new();
    generator.pseudo_legal(&board, &mut moves);

    println!("Initial board:\n{}", board.as_ascii());
    println!("Possible moves: {}", moves.len());

    // print all possible moves
    board.make_move(moves[0].from, moves[0].to);
    moves.clear();
    generator.pseudo_legal(&board, &mut moves);
    println!("After making move {}: {}", moves[0], board.as_ascii());
    println!("Possible moves: {}", moves.len());
}
