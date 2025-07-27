//! Project type detection for auto-generating appropriate hook defaults
//!
//! Detects project type based on presence of common configuration files.

use std::path::Path;

/// Supported project types for auto-detection
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Rust,
    Go,
    Node,
    Python,
    Unknown,
}

impl ProjectType {
    /// Auto-detect project type based on files in current directory
    pub fn auto_detect() -> Self {
        // Check for Rust project
        if Path::new("Cargo.toml").exists() {
            return ProjectType::Rust;
        }

        // Check for Go project
        if Path::new("go.mod").exists() || Path::new("go.sum").exists() {
            return ProjectType::Go;
        }

        // Check for Node.js project
        if Path::new("package.json").exists() {
            return ProjectType::Node;
        }

        // Check for Python project
        if Path::new("requirements.txt").exists()
            || Path::new("pyproject.toml").exists()
            || Path::new("setup.py").exists()
            || Path::new("Pipfile").exists()
        {
            return ProjectType::Python;
        }

        ProjectType::Unknown
    }

    /// Create project type from string
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Some(ProjectType::Rust),
            "go" | "golang" => Some(ProjectType::Go),
            "node" | "nodejs" | "javascript" | "js" | "typescript" | "ts" => {
                Some(ProjectType::Node)
            }
            "python" | "py" => Some(ProjectType::Python),
            _ => None,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ProjectType::Rust => "Rust",
            ProjectType::Go => "Go",
            ProjectType::Node => "Node.js",
            ProjectType::Python => "Python",
            ProjectType::Unknown => "Unknown",
        }
    }

    /// Get recommended pre-commit command for this project type
    pub fn default_pre_commit_command(&self) -> &'static str {
        match self {
            ProjectType::Rust => "cargo fmt --check && cargo clippy -- -D warnings",
            ProjectType::Go => "go fmt ./... && go vet ./...",
            ProjectType::Node => "npm run lint && npm test",
            ProjectType::Python => "black --check . && flake8",
            ProjectType::Unknown => "echo 'Please configure your pre-commit hook in samoid.toml'",
        }
    }

    /// Get optional pre-push command for this project type
    pub fn default_pre_push_command(&self) -> Option<&'static str> {
        match self {
            ProjectType::Rust => Some("cargo test --release"),
            ProjectType::Go => Some("go test ./..."),
            ProjectType::Node => Some("npm test"),
            ProjectType::Python => Some("python -m pytest"),
            ProjectType::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_temp_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();
        temp_dir
    }

    #[test]
    fn test_detect_rust_project() {
        let _temp_dir = setup_temp_dir();
        fs::write("Cargo.toml", "[package]\nname = \"test\"").unwrap();

        assert_eq!(ProjectType::auto_detect(), ProjectType::Rust);
    }

    #[test]
    fn test_detect_go_project() {
        let _temp_dir = setup_temp_dir();
        fs::write("go.mod", "module test").unwrap();

        assert_eq!(ProjectType::auto_detect(), ProjectType::Go);
    }

    #[test]
    fn test_detect_node_project() {
        let _temp_dir = setup_temp_dir();
        fs::write("package.json", "{}").unwrap();

        assert_eq!(ProjectType::auto_detect(), ProjectType::Node);
    }

    #[test]
    fn test_detect_python_project() {
        let _temp_dir = setup_temp_dir();
        fs::write("requirements.txt", "").unwrap();

        assert_eq!(ProjectType::auto_detect(), ProjectType::Python);
    }

    #[test]
    fn test_detect_python_project_pyproject() {
        let _temp_dir = setup_temp_dir();
        fs::write("pyproject.toml", "").unwrap();

        assert_eq!(ProjectType::auto_detect(), ProjectType::Python);
    }

    #[test]
    fn test_detect_unknown_project() {
        let _temp_dir = setup_temp_dir();

        assert_eq!(ProjectType::auto_detect(), ProjectType::Unknown);
    }

    #[test]
    fn test_from_string() {
        assert_eq!(ProjectType::from_string("rust"), Some(ProjectType::Rust));
        assert_eq!(ProjectType::from_string("RUST"), Some(ProjectType::Rust));
        assert_eq!(ProjectType::from_string("rs"), Some(ProjectType::Rust));

        assert_eq!(ProjectType::from_string("go"), Some(ProjectType::Go));
        assert_eq!(ProjectType::from_string("golang"), Some(ProjectType::Go));

        assert_eq!(ProjectType::from_string("node"), Some(ProjectType::Node));
        assert_eq!(ProjectType::from_string("nodejs"), Some(ProjectType::Node));
        assert_eq!(
            ProjectType::from_string("javascript"),
            Some(ProjectType::Node)
        );
        assert_eq!(
            ProjectType::from_string("typescript"),
            Some(ProjectType::Node)
        );

        assert_eq!(
            ProjectType::from_string("python"),
            Some(ProjectType::Python)
        );
        assert_eq!(ProjectType::from_string("py"), Some(ProjectType::Python));

        assert_eq!(ProjectType::from_string("invalid"), None);
    }

    #[test]
    fn test_project_type_names() {
        assert_eq!(ProjectType::Rust.name(), "Rust");
        assert_eq!(ProjectType::Go.name(), "Go");
        assert_eq!(ProjectType::Node.name(), "Node.js");
        assert_eq!(ProjectType::Python.name(), "Python");
        assert_eq!(ProjectType::Unknown.name(), "Unknown");
    }

    #[test]
    fn test_default_commands() {
        assert!(
            ProjectType::Rust
                .default_pre_commit_command()
                .contains("cargo")
        );
        assert!(
            ProjectType::Go
                .default_pre_commit_command()
                .contains("go fmt")
        );
        assert!(
            ProjectType::Node
                .default_pre_commit_command()
                .contains("npm")
        );
        assert!(
            ProjectType::Python
                .default_pre_commit_command()
                .contains("black")
        );

        assert!(
            ProjectType::Rust
                .default_pre_push_command()
                .unwrap()
                .contains("cargo test")
        );
        assert!(
            ProjectType::Go
                .default_pre_push_command()
                .unwrap()
                .contains("go test")
        );
        assert!(
            ProjectType::Node
                .default_pre_push_command()
                .unwrap()
                .contains("npm test")
        );
        assert!(
            ProjectType::Python
                .default_pre_push_command()
                .unwrap()
                .contains("pytest")
        );

        assert!(ProjectType::Unknown.default_pre_push_command().is_none());
    }

    #[test]
    fn test_priority_detection() {
        // Test that Rust takes priority when multiple project files exist
        let _temp_dir = setup_temp_dir();
        fs::write("Cargo.toml", "[package]\nname = \"test\"").unwrap();
        fs::write("package.json", "{}").unwrap();

        // Should detect Rust because it's checked first
        assert_eq!(ProjectType::auto_detect(), ProjectType::Rust);
    }
}
