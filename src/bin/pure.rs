//! Aether Chess Engine - Pure Alpha-Beta variant
//! Baseline implementation without optimizations

use engine::search::SearcherType;
use engine::Engine;
use interface::UciHandler;

fn main() {
    let engine = Engine::with_searcher_type(SearcherType::PureAlphaBeta, 16);
    let mut handler = UciHandler::with_engine(engine);
    handler.run();
}
