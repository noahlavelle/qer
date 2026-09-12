use std::collections::HashMap;

use tokio::sync::Mutex;
use tonic::async_trait;

use crate::engine::{
    JobID, QueueID, ReservationID, WorkerID,
    queue::Queue,
    store::{MetadataStore, PayloadStore, StoreError},
};

struct StoredReservation {
    job_id: JobID,
    worker_id: WorkerID,
}

pub struct InMemoryStoreState {
    queues: HashMap<QueueID, Queue>,
    reservations: HashMap<ReservationID, StoredReservation>,
    payloads: HashMap<JobID, Vec<u8>>,
}

pub struct InMemoryStore {
    state: Mutex<InMemoryStoreState>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(InMemoryStoreState {
                queues: HashMap::new(),
                reservations: HashMap::new(),
                payloads: HashMap::new(),
            }),
        }
    }
}

#[async_trait]
impl MetadataStore for InMemoryStore {
    async fn create_queue(&self, queue_id: QueueID) -> Result<(), StoreError> {
        let mut state = self.state.lock().await;

        if state.queues.contains_key(&queue_id) {
            return Err(StoreError::QueueAlreadyExists(queue_id.into()));
        }

        state.queues.insert(queue_id, Queue::new());

        Ok(())
    }

    async fn enqueue(&self, queue_id: &QueueID, job_id: JobID) -> Result<(), StoreError> {
        let mut state = self.state.lock().await;

        let queue = state
            .queues
            .get_mut(queue_id)
            .ok_or_else(|| StoreError::QueueNotFound(queue_id.as_str().to_owned()))?;

        queue.put(job_id);

        Ok(())
    }

    async fn reserve(
        &self,
        queue_id: &QueueID,
        reservation_id: ReservationID,
        worker_id: WorkerID,
    ) -> Result<Option<JobID>, StoreError> {
        let mut state = self.state.lock().await;

        let queue = state
            .queues
            .get_mut(queue_id)
            .ok_or_else(|| StoreError::QueueNotFound(queue_id.as_str().to_owned()))?;

        let Some(job_id) = queue.reserve() else {
            return Ok(None);
        };

        state.reservations.insert(
            reservation_id,
            StoredReservation {
                job_id: job_id.clone(),
                worker_id,
            },
        );

        Ok(Some(job_id))
    }

    async fn ack(
        &self,
        reservation_id: &ReservationID,
        worker_id: &WorkerID,
    ) -> Result<JobID, StoreError> {
        let mut state = self.state.lock().await;

        let reservation = state
            .reservations
            .get(reservation_id)
            .ok_or_else(|| StoreError::ReservationNotFound(reservation_id.as_str().to_owned()))?;

        if reservation.worker_id != *worker_id {
            return Err(StoreError::AccessDenied);
        }

        let reservation = state.reservations.remove(reservation_id).unwrap();
        Ok(reservation.job_id)
    }
}

#[async_trait]
impl PayloadStore for InMemoryStore {
    async fn put(&self, job_id: JobID, payload: Vec<u8>) -> Result<(), StoreError> {
        let mut state = self.state.lock().await;

        state.payloads.insert(job_id, payload);

        Ok(())
    }

    async fn get(&self, job_id: &JobID) -> Result<Vec<u8>, StoreError> {
        let state = self.state.lock().await;

        let payload = state
            .payloads
            .get(job_id)
            .cloned()
            .ok_or_else(|| StoreError::JobNotFound(job_id.as_str().to_owned()))?;

        Ok(payload)
    }

    async fn delete(&self, job_id: &JobID) -> Result<(), StoreError> {
        let mut state = self.state.lock().await;

        state.payloads.remove(job_id);

        Ok(())
    }
}
