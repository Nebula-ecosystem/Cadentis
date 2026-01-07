use crate::runtime::Runtime;

pub struct RuntimeBuilder {
    enable_io: bool,
    enable_fs: bool,
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self {
            enable_io: false,
            enable_fs: false,
        }
    }

    pub fn enable_io(mut self) -> Self {
        self.enable_io = true;
        self
    }

    pub fn enable_fs(mut self) -> Self {
        self.enable_fs = true;
        self.enable_io = true; // Filesystem support relies on reactor I/O for non-blocking operations.
        self
    }

    pub fn build(self) -> Runtime {
        Runtime::with_features(self.enable_io, self.enable_fs)
    }
}
