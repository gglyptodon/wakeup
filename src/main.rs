use std::process::exit;
use wakeup::{get_args, run};

fn main() {
    if let Err(e) = get_args().and_then(run) {
        eprintln!("An error occured: {}", e);
        exit(1);
    }
}
