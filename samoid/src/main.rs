mod git;
mod hooks;
mod installer;

use installer::install_hooks;

fn main() {
    match install_hooks(None) {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{}", msg);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
