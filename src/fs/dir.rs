use crate::reactor::poller::platform::sys_mkdir;

use std::ffi::CString;
use std::io;
use std::path::{Component, Path, PathBuf};

pub struct Dir {
    path: PathBuf,
}

impl Dir {
    pub async fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        Self::make_directory(path.as_ref())?;

        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }

    pub async fn create_all(path: impl AsRef<Path>) -> io::Result<Self> {
        let target = path.as_ref();

        if target.as_os_str().is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty path"));
        }

        let mut acc = PathBuf::new();

        if target.is_absolute() {
            acc.push(Path::new("/"));
        }

        for component in target.components() {
            match component {
                Component::RootDir => {}
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "parent directory (..) not supported",
                    ));
                }
                Component::Normal(seg) => {
                    acc.push(seg);
                    Self::make_directory(&acc)?;
                }
                _ => {
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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn exists(&self) -> bool {
        self.path.is_dir()
    }

    fn make_directory(path: &Path) -> io::Result<()> {
        let c_path = CString::new(
            path.as_os_str()
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "non UTF-8 path"))?,
        )?;

        let rc = sys_mkdir(c_path.as_ptr(), 0o755);

        if rc < 0 {
            let error = io::Error::last_os_error();

            match error.kind() {
                io::ErrorKind::AlreadyExists => Ok(()),
                _ => Err(error),
            }
        } else {
            Ok(())
        }
    }
}
