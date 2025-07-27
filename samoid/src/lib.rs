//! Samoid - A fast, native Git hooks manager written in Rust
//!
//! Samoid is a reimplementation of Husky, providing efficient Git hook management
//! with minimal dependencies and improved performance.
//!
//! # Example
//!
//! ```no_run
//! use samoid::install_hooks;
//! use samoid::environment::{SystemEnvironment, SystemCommandRunner, SystemFileSystem};
//!
//! let env = SystemEnvironment;
//! let runner = SystemCommandRunner;
//! let fs = SystemFileSystem;
//!
//! match install_hooks(&env, &runner, &fs, None) {
//!     Ok(message) => println!("{}", message),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

/// Environment abstractions for dependency injection and testing
pub mod environment;

/// Git repository operations and configuration
pub mod git;

/// Git hook file creation and management
pub mod hooks;

/// Core installation logic for setting up Git hooks
pub mod installer;

/// Re-export the main installation function for convenience
pub use installer::install_hooks;
