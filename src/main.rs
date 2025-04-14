use movegen::magic_gen;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let constants_path = "movegen/src/magic_constants.rs";

    if args.len() > 1 && args[1] == "generate-magics" {
        println!("Generating magic bitboard constants...");

        if let Err(e) = magic_gen::generate_magic_constants(constants_path) {
            eprintln!("Failed to generate magic constants: {}", e);
            std::process::exit(1);
        }

        println!("Magic constants generation complete!");
    }
}
