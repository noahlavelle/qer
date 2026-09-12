use thiserror::Error;
use tonic::async_trait;

use crate::engine::{JobID, QueueID, ReservationID, WorkerID};

pub mod memory;

#[derive(Error, Debug)]
pub enum StoreError {
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

#[async_trait]
pub trait MetadataStore: Send + Sync {
    async fn create_queue(&self, queue_id: QueueID) -> Result<(), StoreError>;
    async fn enqueue(&self, queue_id: &QueueID, job_id: JobID) -> Result<(), StoreError>;
    async fn reserve(
        &self,
        queue_id: &QueueID,
        reservation_id: ReservationID,
        worker_id: WorkerID,
    ) -> Result<Option<JobID>, StoreError>;
    async fn ack(
        &self,
        reservation_id: &ReservationID,
        worker_id: &WorkerID,
    ) -> Result<JobID, StoreError>;
}

#[async_trait]
pub trait PayloadStore: Send + Sync {
    async fn put(&self, job_id: JobID, payload: Vec<u8>) -> Result<(), StoreError>;
    async fn get(&self, job_id: &JobID) -> Result<Vec<u8>, StoreError>;
    async fn delete(&self, job_id: &JobID) -> Result<(), StoreError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_not_found_includes_the_name() {
        let err = StoreError::QueueNotFound("orders".to_owned());
        assert_eq!(err.to_string(), "queue not found: orders");
    }

    #[test]
    fn queue_already_exists_includes_the_name() {
        let err = StoreError::QueueAlreadyExists("orders".to_owned());
        assert_eq!(err.to_string(), "queue already exists: orders");
    }

    #[test]
    fn access_denied_has_a_static_message() {
        let err = StoreError::AccessDenied;
        assert_eq!(err.to_string(), "access denied");
    }
}
