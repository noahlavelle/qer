use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueueIDError {
    #[error("queue ID cannot be empty")]
    Empty,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct QueueID(String);

impl QueueID {
    pub fn new(value: impl Into<String>) -> Result<Self, QueueIDError> {
        let value = value.into();

        if value.is_empty() {
            return Err(QueueIDError::Empty);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<QueueID> for String {
    fn from(id: QueueID) -> String {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_id_rejects_empty() {
        // QueueID doesn't derive Debug, so unwrap_err() can't be used here.
        let err = match QueueID::new("") {
            Err(e) => e,
            Ok(_) => panic!("expected an error"),
        };
        assert!(matches!(err, QueueIDError::Empty));
    }

    #[test]
    fn queue_id_accepts_nonempty() {
        let id = QueueID::new("orders").unwrap();
        assert_eq!(id.as_str(), "orders");
    }
}
