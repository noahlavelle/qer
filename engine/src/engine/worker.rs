use crate::engine::error::WorkerIDError;

#[derive(Clone, Eq, PartialEq)]
pub struct WorkerID(String);

impl WorkerID {
    pub fn new(value: impl Into<String>) -> Result<Self, WorkerIDError> {
        let value = value.into();

        if value.is_empty() {
            return Err(WorkerIDError::Empty);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
