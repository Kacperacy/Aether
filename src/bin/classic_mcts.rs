//! Aether Chess Engine - Classic MCTS variant
//! Monte Carlo Tree Search with random playouts (baseline)

use engine::search::SearcherType;
use engine::Engine;
use interface::UciHandler;

fn main() {
    let engine = Engine::with_searcher_type(SearcherType::ClassicMcts, 16);
    let mut handler = UciHandler::with_engine(engine);
    handler.run();
}
