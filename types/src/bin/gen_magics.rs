#[cfg(feature = "codegen")]
fn main() -> std::io::Result<()> {
    use std::path::PathBuf;

    println!("====================================");
    println!("  Magic Bitboard Generator");
    println!("====================================\n");

    // Determine output path
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let output_path = PathBuf::from(&manifest_dir)
        .join("src")
        .join("attacks")
        .join("magic_constants.rs");

    println!("Output: {}\n", output_path.display());

    // Check if file exists
    if output_path.exists() {
        println!("⚠️  File already exists!");
        print!("Do you want to regenerate? [y/N] ");

        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
        println!();
    }

    // Generate
    use aether_types::attacks::codegen;

    println!("Generating magic bitboards...");
    println!("This will take 10-30 seconds...\n");

    codegen::generate_magic_constants(&output_path)?;

    println!("\n====================================");
    println!("  ✅ Generation Complete!");
    println!("====================================\n");

    Ok(())
}

#[cfg(not(feature = "codegen"))]
fn main() {
    eprintln!("Error: This binary requires the 'codegen' feature");
    eprintln!("Run with: cargo run -p aether-types --features codegen --bin gen_magics");
    std::process::exit(1);
}
