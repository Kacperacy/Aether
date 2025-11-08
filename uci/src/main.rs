use uci::run_uci_loop;

fn main() {
    if let Err(e) = run_uci_loop() {
        eprintln!("UCI error: {}", e);
        std::process::exit(1);
    }
}
