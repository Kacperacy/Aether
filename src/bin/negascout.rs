//! Aether Chess Engine - NegaScout variant
//! Principal Variation Search algorithm (baseline)

use engine::search::SearcherType;
use engine::Engine;
use interface::UciHandler;

fn main() {
    let engine = Engine::with_searcher_type(SearcherType::NegaScout, 128);
    let mut handler = UciHandler::with_engine(engine);
    handler.run();
}
