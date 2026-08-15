//! Aether Chess Engine
//!
//! A modular chess engine written in Rust
//! Run without arguments to start UCI mode

use interface::UciHandler;
use std::env;

fn print_usage() {
    println!("Aether Chess Engine");
    println!();
    println!("Usage:");
    println!("  aether              Start UCI mode (for GUI integration)");
    println!(
        "  aether bench [d]    Fixed-workload search benchmark (default depth {})",
        engine::bench::DEFAULT_BENCH_DEPTH
    );
    println!("  aether --help       Show this help message");
    println!("  aether --version    Show version information");
    println!();
    println!("Development commands:");
    println!("  cargo run -p attacks --features codegen --bin gen_magics");
    println!("                      Generate magic bitboards");
    println!("  cargo test --workspace");
    println!("                      Run all tests");
}

/// Run the search benchmark and print the regression figures.
///
/// The node count is deterministic for a given binary and depth; treat any
/// unexplained change to it as a search behaviour change.
fn run_bench(depth: u8) {
    let result = engine::bench::run(depth);

    println!();
    println!("Positions: {}", result.positions);
    println!("Depth: {}", result.depth);
    println!("Nodes: {}", result.nodes);
    println!("Time: {:?}", result.elapsed);
    println!("NPS: {}", result.nps());
}

fn print_version() {
    println!("Aether Chess Engine v0.1.0");
    println!("By Kacper Maciołek");
    println!("https://github.com/Kacperacy/Aether");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--version" | "-v" => {
                print_version();
                return;
            }
            "bench" => {
                let depth = args
                    .get(2)
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(engine::bench::DEFAULT_BENCH_DEPTH);
                run_bench(depth);
                return;
            }
            "generate-magics" => {
                println!("This command has moved. Use:");
                println!("  cargo run -p attacks --features codegen --bin gen_magics");
                return;
            }
            _ => {
                println!("Unknown option: {}", args[1]);
                print_usage();
                return;
            }
        }
    }

    // Default: run UCI mode
    let mut handler = UciHandler::new();
    handler.run();
}
