use crate::reactor::poller::platform::sys_mkdir;

use std::ffi::CString;
use std::io;
use std::path::{Component, Path, PathBuf};

/// A filesystem directory handle.
///
/// `Dir` provides asynchronous-friendly directory creation utilities.
/// While directory creation itself is not awaitable at the OS level,
/// these methods are exposed as async for API consistency with the
/// rest of the filesystem module.
pub struct Dir {
    /// Path to the directory.
    path: PathBuf,
}

impl Dir {
    /// Creates a single directory.
    ///
    /// This is the async equivalent of `std::fs::create_dir`.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    pub async fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::make_directory(path.as_ref())?;

        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }

    /// Recursively creates a directory and all of its parent components.
    ///
    /// This is the async equivalent of `std::fs::create_dir_all`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - the path is empty,
    /// - a parent directory (`..`) is encountered,
    /// - a component is invalid or unsupported,
    /// - a directory cannot be created.
    pub async fn create_all(path: impl AsRef<Path>) -> io::Result<Self> {
        let target = path.as_ref();

        if target.as_os_str().is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty path"));
        }

        let mut acc = PathBuf::new();
        let mut components = target.components();

        if let Some(first) = components.next() {
            match first {
                Component::Prefix(p) => {
                    acc.push(p.as_os_str());

                    if let Some(Component::RootDir) = components.next() {
                        acc.push(Path::new("/"));
                    }
                }
                Component::RootDir => {
                    acc.push(Path::new("/"));
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "parent directory (..) not supported",
                    ));
                }
                Component::Normal(seg) => {
                    acc.push(seg);
                    Self::make_directory(&acc).or_else(|e| {
                        if e.kind() == io::ErrorKind::AlreadyExists && acc.is_dir() {
                            Ok(())
                        } else {
                            Err(e)
                        }
                    })?;
                }
            }
        }

        for component in components {
            match component {
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "parent directory (..) not supported",
                    ));
                }
                Component::Normal(seg) => {
                    acc.push(seg);

                    match Self::make_directory(&acc) {
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::AlreadyExists && acc.is_dir() => {}
                        Err(e) => return Err(e),
                    }
                }
                Component::RootDir => {}
                Component::Prefix(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "unsupported path component",
                    ));
                }
            }
        }

        Ok(Self {
            path: target.to_path_buf(),
        })
    }

    /// Returns the path of this directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns `true` if the directory exists on disk.
    pub fn exists(&self) -> bool {
        self.path.is_dir()
    }

    /// Creates a directory at the specified path.
    fn make_directory(path: &Path) -> io::Result<()> {
        let c_path = CString::new(
            path.as_os_str()
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "non UTF-8 path"))?,
        )?;

        let rc = sys_mkdir(c_path.as_ptr(), 0o755);

        #[cfg(windows)]
        if rc == u64::MAX {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }

        #[cfg(unix)]
        if rc < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
