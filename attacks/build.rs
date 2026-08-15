use std::path::PathBuf;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let constants_path = PathBuf::from(&manifest_dir)
        .join("src")
        .join("magic_constants.rs");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/codegen.rs");

    if !constants_path.exists() {
        panic!(
            "Magic constants file not found at: {}\n\
             Please run: cargo run -p attacks --features codegen --bin gen_magics",
            constants_path.display()
        );
    }
}
