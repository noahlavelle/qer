use std::{collections::HashMap};
use tokio::sync::Mutex;

mod error;
mod job;
mod queue;
mod reservation;
mod worker;

pub use error::EngineError;
pub use job::{Job, JobID};
pub use queue::QueueID;
pub use reservation::{Reservation, ReservationID};
pub use worker::WorkerID;

use crate::engine::queue::Queue;

struct EngineState {
    queues: HashMap<QueueID, Queue>,
    reservations: HashMap<ReservationID, Reservation>,
}

pub struct Engine {
    state: Mutex<EngineState>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(EngineState {
                queues: HashMap::new(),
                reservations: HashMap::new(),
            }),
        }
    }

    pub async fn create_queue(
        &self,
        queue_id: QueueID
    ) -> Result<(), EngineError> {
        let mut state = self.state.lock().await;

        if state.queues.contains_key(&queue_id) {
            return Err(EngineError::QueueAlreadyExists(
                queue_id.as_str().to_owned(),
            ));
        }

        state.queues.insert(queue_id, queue::Queue::new());

        Ok(())
    }

    pub async fn put(
        &self,
        queue_id: &QueueID,
        payload: Vec<u8>
    ) -> Result<JobID, EngineError> {
        let mut state = self.state.lock().await;

        let queue = state
            .queues
            .get_mut(queue_id)
            .ok_or_else(|| {
                EngineError::QueueNotFound(queue_id.as_str().to_owned())
            })?;

        let job_id = JobID::new();
        let job = Job {
            id: job_id.clone(),
            queue_id: queue_id.clone(),
            payload: payload,
        };

        queue.put(job);

        Ok(job_id)
    }

    pub async fn reserve(
        &self,
        queue_id: &QueueID,
        worker_id: &WorkerID
    ) -> Result<Option<Reservation>, EngineError> {
        let mut state = self.state.lock().await;

        let queue = state
            .queues
            .get_mut(queue_id)
            .ok_or_else(|| {
                EngineError::QueueNotFound(queue_id.as_str().to_owned())
            })?;

        let Some(job) = queue.reserve() else {
            return Ok(None);
        };

        let reservation = Reservation {
            id: ReservationID::generate(),
            worker_id: worker_id.clone(),
            job: job,
        };

        state.reservations.insert(
            reservation.id.clone(),
            reservation.clone(),
        );

        Ok(Some(reservation))
    }

    pub async fn ack(
        &self,
        reservation_id: &ReservationID,
    ) -> Result<(), EngineError> {
        let mut state = self.state.lock().await;

        if state.reservations.remove(reservation_id).is_none() {
            return Err(EngineError::ReservationNotFound(
                reservation_id.as_str().to_owned()
            ));
        }

        Ok(())
    }
}
