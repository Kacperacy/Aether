use board::Board;

fn perft(board: &mut Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut moves = Vec::new();
    movegen::legal(board, &mut moves);

    if depth == 1 {
        return moves.len() as u64;
    }

    let mut nodes = 0u64;
    for mv in moves {
        board.make_move(&mv).unwrap();
        nodes += perft(board, depth - 1);
        board.unmake_move(&mv).unwrap();
    }
    nodes
}

fn main() {
    let mut board = Board::starting_position().unwrap();

    let mut moves = Vec::new();
    movegen::legal(&board, &mut moves);
    moves.sort_by(|a, b| format!("{}", a).cmp(&format!("{}", b)));

    let mut total = 0u64;
    for mv in &moves {
        board.make_move(mv).unwrap();
        let count = perft(&mut board, 2);
        board.unmake_move(mv).unwrap();
        println!("{}: {}", mv, count);
        total += count;
    }
    println!("\nTotal: {}", total);
    println!("Moves: {}", moves.len());
}
