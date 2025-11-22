use std::path::PathBuf;

fn main() {
    #[cfg(feature = "codegen")]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let output_path = PathBuf::from(&manifest_dir)
            .join("src")
            .join("attacks")
            .join("magic_constants.rs");

        let should_generate = !output_path.exists() || {
            let build_script = PathBuf::from(&manifest_dir).join("build.rs");
            let build_time = std::fs::metadata(&build_script)
                .and_then(|m| m.modified())
                .ok();
            let output_time = std::fs::metadata(&output_path)
                .and_then(|m| m.modified())
                .ok();

            match (build_time, output_time) {
                (Some(b), Some(o)) => b > o,
                _ => true,
            }
        };

        if should_generate {
            println!("cargo:warning=Generating magic bitboard constants...");
            println!("cargo:warning=This may take 10-30 seconds...");

            // Note: This would require the codegen module to be available at build time
            // In practice, you might want to use a separate binary or keep it manual
            // For now, this is a template showing how it COULD work

            println!("cargo:warning=To generate manually, run:");
            println!(
                "cargo:warning=  cargo run -p aether-types --features codegen --bin gen_magics"
            );
        }

        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-changed=src/attacks/codegen.rs");
    }

    #[cfg(not(feature = "codegen"))]
    {
        // When codegen feature is not enabled, just verify the file exists
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let constants_path = PathBuf::from(&manifest_dir)
            .join("src")
            .join("attacks")
            .join("magic_constants.rs");

        if !constants_path.exists() {
            panic!(
                "Magic constants file not found at: {}\n\
                 Please run: cargo run -p types --features codegen --bin gen_magics",
                constants_path.display()
            );
        }
    }
}
