use super::Runtime;

pub struct RuntimeBuilder {}

impl RuntimeBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self) -> Runtime {
        Runtime::new()
    }
}
