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
}

#[derive(Error, Debug)]
pub enum QueueIDError {
    #[error("queue ID cannot be empty")]
    Empty,
}

#[derive(Error, Debug)]
pub enum ReservationIDError {
    #[error("reservation ID cannot be empty")]
    Empty,
}

#[derive(Error, Debug)]
pub enum WorkerIDError {
    #[error("worker ID cannot be empty")]
    Empty,
}
