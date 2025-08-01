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
        Self::auto_detect_in_path(".")
    }

    /// Auto-detect project type based on files in specified directory
    pub fn auto_detect_in_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        // Check for Rust project
        if path.join("Cargo.toml").exists() {
            return ProjectType::Rust;
        }

        // Check for Go project
        if path.join("go.mod").exists() || path.join("go.sum").exists() {
            return ProjectType::Go;
        }

        // Check for Node.js project
        if path.join("package.json").exists() {
            return ProjectType::Node;
        }

        // Check for Python project
        if path.join("requirements.txt").exists()
            || path.join("pyproject.toml").exists()
            || path.join("setup.py").exists()
            || path.join("Pipfile").exists()
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
            ProjectType::Unknown => "echo 'Please configure your pre-commit hook in samoyed.toml'",
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
#[path = "unit_tests/project_tests.rs"]
mod tests;
