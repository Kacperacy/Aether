use movegen::magic_gen;

fn main() {
    // Generate magic constants into this crate's source tree in a reproducible location.
    let constants_path = format!("{}/src/magic_constants.rs", env!("CARGO_MANIFEST_DIR"));

    println!(
        "Generating magic bitboard constants into: {}",
        constants_path
    );
    if let Err(e) = magic_gen::generate_magic_constants(&constants_path) {
        eprintln!("Failed to generate magic constants: {e}");
        std::process::exit(1);
    }

    println!("Magic constants generation complete!");
}
