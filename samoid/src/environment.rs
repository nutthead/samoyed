use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Trait for abstracting environment variable operations
pub trait Environment {
    fn get_var(&self, key: &str) -> Option<String>;
    fn set_var(&mut self, key: &str, value: &str);
    fn remove_var(&mut self, key: &str);
    fn current_dir(&self) -> io::Result<PathBuf>;
    fn set_current_dir(&mut self, path: &Path) -> io::Result<()>;
}

/// Trait for abstracting command execution
pub trait CommandRunner {
    fn run_command(&self, program: &str, args: &[&str]) -> io::Result<Output>;
}

/// Trait for abstracting file system operations
pub trait FileSystem {
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn write(&self, path: &Path, contents: &str) -> io::Result<()>;
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn set_permissions(&self, path: &Path, mode: u32) -> io::Result<()>;
}

/// Production implementation that interacts with the real system
pub struct SystemEnvironment;

impl Environment for SystemEnvironment {
    fn get_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn set_var(&mut self, key: &str, value: &str) {
        unsafe {
            std::env::set_var(key, value);
        }
    }

    fn remove_var(&mut self, key: &str) {
        unsafe {
            std::env::remove_var(key);
        }
    }

    fn current_dir(&self) -> io::Result<PathBuf> {
        std::env::current_dir()
    }

    fn set_current_dir(&mut self, path: &Path) -> io::Result<()> {
        std::env::set_current_dir(path)
    }
}

pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run_command(&self, program: &str, args: &[&str]) -> io::Result<Output> {
        Command::new(program).args(args).output()
    }
}

pub struct SystemFileSystem;

impl FileSystem for SystemFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
        fs::write(path, contents)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    #[cfg(unix)]
    fn set_permissions(&self, path: &Path, mode: u32) -> io::Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(mode);
        fs::set_permissions(path, permissions)
    }

    #[cfg(not(unix))]
    fn set_permissions(&self, _path: &Path, _mode: u32) -> io::Result<()> {
        // On non-Unix systems, we'll just return Ok
        // In production, you might want to handle Windows permissions differently
        Ok(())
    }
}

/// Mock implementation for testing
pub mod mocks {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Mutex;

    pub struct MockEnvironment {
        vars: Arc<Mutex<HashMap<String, String>>>,
        current_dir: Arc<Mutex<PathBuf>>,
    }

    impl MockEnvironment {
        pub fn new() -> Self {
            Self {
                vars: Arc::new(Mutex::new(HashMap::new())),
                current_dir: Arc::new(Mutex::new(PathBuf::from("/tmp/test"))),
            }
        }

        pub fn with_var(self, key: &str, value: &str) -> Self {
            self.vars
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            self
        }

        pub fn with_current_dir(self, path: PathBuf) -> Self {
            *self.current_dir.lock().unwrap() = path;
            self
        }
    }

    impl Environment for MockEnvironment {
        fn get_var(&self, key: &str) -> Option<String> {
            self.vars.lock().unwrap().get(key).cloned()
        }

        fn set_var(&mut self, key: &str, value: &str) {
            self.vars
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
        }

        fn remove_var(&mut self, key: &str) {
            self.vars.lock().unwrap().remove(key);
        }

        fn current_dir(&self) -> io::Result<PathBuf> {
            Ok(self.current_dir.lock().unwrap().clone())
        }

        fn set_current_dir(&mut self, path: &Path) -> io::Result<()> {
            *self.current_dir.lock().unwrap() = path.to_path_buf();
            Ok(())
        }
    }

    pub struct MockCommandRunner {
        responses: Arc<Mutex<HashMap<String, io::Result<Output>>>>,
    }

    impl MockCommandRunner {
        pub fn new() -> Self {
            Self {
                responses: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub fn with_response(
            self,
            command: &str,
            args: &[&str],
            output: io::Result<Output>,
        ) -> Self {
            let key = format!("{} {}", command, args.join(" "));
            self.responses.lock().unwrap().insert(key, output);
            self
        }
    }

    impl CommandRunner for MockCommandRunner {
        fn run_command(&self, program: &str, args: &[&str]) -> io::Result<Output> {
            let key = format!("{} {}", program, args.join(" "));
            if let Some(result) = self.responses.lock().unwrap().get(&key) {
                match result {
                    Ok(output) => Ok(Output {
                        status: output.status,
                        stdout: output.stdout.clone(),
                        stderr: output.stderr.clone(),
                    }),
                    Err(e) => Err(io::Error::new(e.kind(), e.to_string())),
                }
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "Command not found"))
            }
        }
    }

    pub struct MockFileSystem {
        files: Arc<Mutex<HashMap<PathBuf, String>>>,
        directories: Arc<Mutex<Vec<PathBuf>>>,
    }

    impl MockFileSystem {
        pub fn new() -> Self {
            Self {
                files: Arc::new(Mutex::new(HashMap::new())),
                directories: Arc::new(Mutex::new(Vec::new())),
            }
        }

        pub fn with_file(self, path: impl Into<PathBuf>, contents: &str) -> Self {
            self.files
                .lock()
                .unwrap()
                .insert(path.into(), contents.to_string());
            self
        }

        pub fn with_directory(self, path: impl Into<PathBuf>) -> Self {
            self.directories.lock().unwrap().push(path.into());
            self
        }
    }

    impl FileSystem for MockFileSystem {
        fn exists(&self, path: &Path) -> bool {
            let files = self.files.lock().unwrap();
            let dirs = self.directories.lock().unwrap();

            files.contains_key(path) || dirs.iter().any(|d| d == path || path.starts_with(d))
        }

        fn create_dir_all(&self, path: &Path) -> io::Result<()> {
            self.directories.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }

        fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), contents.to_string());
            Ok(())
        }

        fn read_to_string(&self, path: &Path) -> io::Result<String> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "File not found"))
        }

        fn set_permissions(&self, _path: &Path, _mode: u32) -> io::Result<()> {
            // Mock implementation doesn't need to do anything
            Ok(())
        }
    }
}
