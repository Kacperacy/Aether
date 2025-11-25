//! Aether Chess Engine
//!
//! A modular chess engine written in Rust.
//! Run without arguments to start UCI mode.

use interface::UciHandler;
use std::env;

fn print_usage() {
    println!("Aether Chess Engine");
    println!();
    println!("Usage:");
    println!("  aether              Start UCI mode (for GUI integration)");
    println!("  aether --help       Show this help message");
    println!("  aether --version    Show version information");
    println!();
    println!("Development commands:");
    println!("  cargo run -p aether-core --features codegen --bin gen_magics");
    println!("                      Generate magic bitboards");
    println!("  cargo test --workspace");
    println!("                      Run all tests");
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
            "generate-magics" => {
                println!("This command has moved. Use:");
                println!("  cargo run -p aether-core --features codegen --bin gen_magics");
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
