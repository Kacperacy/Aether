use aether_types::Square;
use board::Board;

fn main() {
    let mut board =
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

    board.print();
    board.make_move(Square::D2, Square::D4);
    board.print();
}
