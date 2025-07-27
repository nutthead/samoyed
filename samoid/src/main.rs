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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_main_with_husky_disabled() {
        // Set HUSKY=0 to test the skip path
        unsafe {
            env::set_var("HUSKY", "0");
        }

        // This test verifies that main() can handle the skip case
        // We can't easily test the actual main() function output without
        // subprocess testing, but we can test the underlying logic
        let result = install_hooks(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HUSKY=0 skip install");

        unsafe {
            env::remove_var("HUSKY");
        }
    }

    #[test]
    fn test_main_with_error_case() {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // This should fail because there's no .git directory
        let result = install_hooks(None);
        assert!(result.is_err());
    }

    #[test]
    fn test_main_success_path() {
        let temp_dir = TempDir::new().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        // Create a git repository
        std::fs::create_dir(".git").unwrap();
        Command::new("git").arg("init").output().ok();

        // This should succeed if git is available
        let result = install_hooks(None);
        if Command::new("git").arg("--version").output().is_ok() {
            // If git is available, installation should succeed
            assert!(result.is_ok());
            let msg = result.unwrap();
            // The success case returns empty string
            assert!(msg.is_empty());
        }
    }
}
