//! Aether Chess Engine - MTD(f) variant
//! Memory-enhanced Test Driver algorithm

use engine::search::SearcherType;
use engine::Engine;
use interface::UciHandler;

fn main() {
    let engine = Engine::with_searcher_type(SearcherType::Mtdf, 128);
    let mut handler = UciHandler::with_engine(engine);
    handler.run();
}
