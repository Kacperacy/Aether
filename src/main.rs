use std::env;

fn print_usage() {
    println!("Aether workspace entry point");
    println!("Common commands:");
    println!("  - Generate magic bitboards:");
    println!("      cargo run -p movegen --features codegen --bin gen_magics");
    println!("  - Run perft tests:");
    println!("      cargo test -p perft");
    println!("  - Build and test the whole workspace:");
    println!("      cargo build --workspace");
    println!("      cargo test --workspace");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "generate-magics" {
        println!(
            "This command has moved. Use the dedicated generator binary in the movegen crate:\n  cargo run -p movegen --features codegen --bin gen_magics"
        );
        return;
    }

    // Default: show help/usage to guide users running the root binary directly
    print_usage();
}
