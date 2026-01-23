//! Aether Chess Engine - MCTS variant
//! Monte Carlo Tree Search algorithm

use engine::search::SearcherType;
use engine::Engine;
use interface::UciHandler;

fn main() {
    let engine = Engine::with_searcher_type(SearcherType::Mcts, 16);
    let mut handler = UciHandler::with_engine(engine);
    handler.run();
}
