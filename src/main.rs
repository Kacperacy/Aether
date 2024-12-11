use aether::board::Board;

fn main() {
    let mut board = Board::init();
    board.print();
    let _ = board.generate_possible_moves();
    board.set_fen("rnbqkbnr/pppp1ppp/8/4p3/3Q4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1");
    board.print();
    board.generate_possible_moves();
}
