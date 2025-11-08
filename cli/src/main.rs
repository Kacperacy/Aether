use cli::ChessCLI;

fn main() {
    let mut cli = ChessCLI::new().expect("Failed to initialize CLI");

    if let Err(e) = cli.run_repl() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
