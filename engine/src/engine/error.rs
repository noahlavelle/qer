use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("queue not found: {0}")]
    QueueNotFound(String),
    #[error("job not found: {0}")]
    JobNotFound(String),
    #[error("job is not reserved: {0}")]
    JobNotReserved(String),
    #[error("job is already reserved: {0}")]
    JobAlreadyReserved(String),
    #[error("reservation not found: {0}")]
    ReservationNotFound(String),
    #[error("queue is empty")]
    QueueEmpty,
    #[error("queue already exists: {0}")]
    QueueAlreadyExists(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("access denied")]
    AccessDenied,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_not_found_includes_the_name() {
        let err = EngineError::QueueNotFound("orders".to_owned());
        assert_eq!(err.to_string(), "queue not found: orders");
    }

    #[test]
    fn queue_already_exists_includes_the_name() {
        let err = EngineError::QueueAlreadyExists("orders".to_owned());
        assert_eq!(err.to_string(), "queue already exists: orders");
    }

    #[test]
    fn access_denied_has_a_static_message() {
        let err = EngineError::AccessDenied;
        assert_eq!(err.to_string(), "access denied");
    }
}
