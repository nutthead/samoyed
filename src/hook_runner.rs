//! Deprecated samoyed-hook Shim Binary
//!
//! **DEPRECATION NOTICE**: This binary is deprecated and will be removed after September 1, 2025.
//!
//! This is now a thin shim that delegates all calls to the new unified `samoyed hook` command.
//!
//! # Migration Path
//!
//! To migrate from `samoyed-hook` to the new unified architecture:
//! 1. Run `samoyed init -f _` to regenerate hook files with the new command structure
//! 2. Hook files will be updated from `exec samoyed-hook ...` to `exec samoyed hook ...`
//!
//! # Legacy Support
//!
//! This shim provides temporary backward compatibility for existing installations.
//! All functionality has been moved to the main `samoyed` binary under the `hook` subcommand.

use std::env;
use std::process::{self, Command};

#[cfg(not(tarpaulin_include))]
fn main() {
    // Print deprecation warning to stderr
    eprintln!(
        "⚠️  WARNING: samoyed-hook is deprecated and will be removed after September 1, 2025."
    );
    eprintln!("   To migrate, run: samoyed init -f _");
    eprintln!("   This will update your hook files to use the new 'samoyed hook' command.");
    eprintln!();

    // Get command line arguments, skipping the program name
    let args: Vec<String> = env::args().skip(1).collect();

    // Delegate to 'samoyed hook' with all the same arguments
    let mut cmd = Command::new("samoyed");
    cmd.arg("hook");
    cmd.args(&args);

    // Execute and exit with the same code
    match cmd.status() {
        Ok(status) => {
            let exit_code = status.code().unwrap_or(1);
            process::exit(exit_code);
        }
        Err(e) => {
            eprintln!("samoyed-hook: Failed to execute 'samoyed hook': {e}");
            eprintln!("samoyed-hook: Make sure 'samoyed' is installed and in your PATH");
            process::exit(127); // Command not found
        }
    }
}
