use aether::board::Board;

fn main() {
    let mut board = Board::init();
    board.print();
    let _ = board.generate_possible_moves();
    board.set_fen("rnbqkbnr/pppp1ppp/8/4q3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1");
    board.print();
    board.generate_possible_moves();
}
