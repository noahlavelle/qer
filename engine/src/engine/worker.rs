use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkerIDError {
    #[error("worker ID cannot be empty")]
    Empty,
}

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

    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<WorkerID> for String {
    fn from(id: WorkerID) -> String {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_empty() {
        // WorkerID doesn't derive Debug, so unwrap_err() can't be used here.
        let err = match WorkerID::new("") {
            Err(e) => e,
            Ok(_) => panic!("expected an error"),
        };
        assert!(matches!(err, WorkerIDError::Empty));
    }

    #[test]
    fn new_accepts_nonempty() {
        let id = WorkerID::new("worker-1").unwrap();
        assert_eq!(id.as_str(), "worker-1");
    }

    #[test]
    fn equality_is_by_value() {
        let a = WorkerID::new("worker-1").unwrap();
        let b = WorkerID::new("worker-1").unwrap();
        let c = WorkerID::new("worker-2").unwrap();

        assert!(a == b);
        assert!(a != c);
    }

    #[test]
    fn clone_preserves_value() {
        let a = WorkerID::new("worker-1").unwrap();
        let cloned = a.clone();
        assert_eq!(a.as_str(), cloned.as_str());
    }
}
