use aether::board::Board;

fn main() {
    let mut board = Board::init();
    board.print();
    board.make_move("e2e4");
    board.print();
}
