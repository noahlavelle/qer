use std::collections::HashMap;
use tokio::sync::Mutex;

mod error;
mod job;
mod queue;
mod reservation;
mod worker;

pub use error::EngineError;
pub use job::{Job, JobID};
pub use queue::{QueueID, QueueIDError};
pub use reservation::{Reservation, ReservationID, ReservationIDError};
pub use worker::{WorkerID, WorkerIDError};

use queue::Queue;

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

    pub async fn create_queue(&self, queue_id: QueueID) -> Result<(), EngineError> {
        let mut state = self.state.lock().await;

        if state.queues.contains_key(&queue_id) {
            return Err(EngineError::QueueAlreadyExists(queue_id.into()));
        }

        state.queues.insert(queue_id, Queue::new());

        Ok(())
    }

    pub async fn put(&self, queue_id: QueueID, payload: Vec<u8>) -> Result<JobID, EngineError> {
        let mut state = self.state.lock().await;

        let queue = state
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| EngineError::QueueNotFound(queue_id.as_str().to_owned()))?;

        let job_id = JobID::generate();
        let job = Job {
            id: job_id.clone(),
            queue_id,
            payload,
        };

        queue.put(job);

        Ok(job_id)
    }

    pub async fn reserve(
        &self,
        queue_id: QueueID,
        worker_id: WorkerID,
    ) -> Result<Option<Reservation>, EngineError> {
        let mut state = self.state.lock().await;

        let queue = state
            .queues
            .get_mut(&queue_id)
            .ok_or_else(|| EngineError::QueueNotFound(queue_id.into()))?;

        let Some(job) = queue.reserve() else {
            return Ok(None);
        };

        let reservation = Reservation {
            id: ReservationID::generate(),
            worker_id,
            job,
        };

        state
            .reservations
            .insert(reservation.id.clone(), reservation.clone());

        Ok(Some(reservation))
    }

    pub async fn ack(
        &self,
        reservation_id: &ReservationID,
        worker_id: &WorkerID,
    ) -> Result<(), EngineError> {
        let mut state = self.state.lock().await;

        let reservation = state
            .reservations
            .get(reservation_id)
            .ok_or_else(|| EngineError::ReservationNotFound(reservation_id.as_str().to_owned()))?;

        if reservation.worker_id != *worker_id {
            return Err(EngineError::AccessDenied);
        }

        state.reservations.remove(reservation_id);

        Ok(())
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queue_id(name: &str) -> QueueID {
        QueueID::new(name).unwrap()
    }

    fn worker_id(name: &str) -> WorkerID {
        WorkerID::new(name).unwrap()
    }

    /// Like `Result::unwrap_err`, but doesn't require the `Ok` variant to be `Debug`
    /// (several engine ID types intentionally don't derive it).
    fn expect_err<T, E>(result: Result<T, E>) -> E {
        match result {
            Err(e) => e,
            Ok(_) => panic!("expected an error"),
        }
    }

    #[tokio::test]
    async fn create_queue_succeeds() {
        let engine = Engine::new();
        assert!(engine.create_queue(queue_id("q")).await.is_ok());
    }

    #[tokio::test]
    async fn create_queue_rejects_duplicates() {
        let engine = Engine::new();
        engine.create_queue(queue_id("q")).await.unwrap();

        let err = engine.create_queue(queue_id("q")).await.unwrap_err();
        assert!(matches!(err, EngineError::QueueAlreadyExists(name) if name == "q"));
    }

    #[tokio::test]
    async fn put_into_unknown_queue_errors() {
        let engine = Engine::new();
        let result = engine.put(queue_id("missing"), vec![1, 2, 3]).await;
        let err = expect_err(result);
        assert!(matches!(err, EngineError::QueueNotFound(name) if name == "missing"));
    }

    #[tokio::test]
    async fn put_returns_unique_job_ids() {
        let engine = Engine::new();
        engine.create_queue(queue_id("q")).await.unwrap();

        let a = engine.put(queue_id("q"), vec![1]).await.unwrap();
        let b = engine.put(queue_id("q"), vec![2]).await.unwrap();

        assert_ne!(a.as_str(), b.as_str());
    }

    #[tokio::test]
    async fn reserve_from_unknown_queue_errors() {
        let engine = Engine::new();
        let result = engine.reserve(queue_id("missing"), worker_id("w1")).await;
        let err = expect_err(result);
        assert!(matches!(err, EngineError::QueueNotFound(name) if name == "missing"));
    }

    #[tokio::test]
    async fn reserve_from_empty_queue_returns_none() {
        let engine = Engine::new();
        engine.create_queue(queue_id("q")).await.unwrap();

        let reservation = engine
            .reserve(queue_id("q"), worker_id("w1"))
            .await
            .unwrap();
        assert!(reservation.is_none());
    }

    #[tokio::test]
    async fn reserve_returns_the_put_job_with_worker_attached() {
        let engine = Engine::new();
        engine.create_queue(queue_id("q")).await.unwrap();
        let job_id = engine
            .put(queue_id("q"), b"payload".to_vec())
            .await
            .unwrap();

        let reservation = engine
            .reserve(queue_id("q"), worker_id("w1"))
            .await
            .unwrap()
            .expect("expected a reservation");

        assert_eq!(reservation.job.id.as_str(), job_id.as_str());
        assert_eq!(reservation.job.payload, b"payload");
        assert!(reservation.worker_id == worker_id("w1"));
    }

    #[tokio::test]
    async fn reserve_does_not_return_the_same_job_twice() {
        let engine = Engine::new();
        engine.create_queue(queue_id("q")).await.unwrap();
        engine.put(queue_id("q"), vec![1]).await.unwrap();

        let first = engine
            .reserve(queue_id("q"), worker_id("w1"))
            .await
            .unwrap();
        let second = engine
            .reserve(queue_id("q"), worker_id("w2"))
            .await
            .unwrap();

        assert!(first.is_some());
        assert!(second.is_none());
    }

    #[tokio::test]
    async fn ack_removes_the_reservation() {
        let engine = Engine::new();
        engine.create_queue(queue_id("q")).await.unwrap();
        engine.put(queue_id("q"), vec![1]).await.unwrap();
        let reservation = engine
            .reserve(queue_id("q"), worker_id("w1"))
            .await
            .unwrap()
            .unwrap();

        assert!(engine.ack(&reservation.id, &worker_id("w1")).await.is_ok());
    }

    #[tokio::test]
    async fn ack_unknown_reservation_errors() {
        let engine = Engine::new();
        let err = engine
            .ack(&ReservationID::generate(), &worker_id("w1"))
            .await
            .unwrap_err();
        assert!(matches!(err, EngineError::ReservationNotFound(_)));
    }

    #[tokio::test]
    async fn ack_by_the_wrong_worker_is_denied() {
        let engine = Engine::new();
        engine.create_queue(queue_id("q")).await.unwrap();
        engine.put(queue_id("q"), vec![1]).await.unwrap();
        let reservation = engine
            .reserve(queue_id("q"), worker_id("w1"))
            .await
            .unwrap()
            .unwrap();

        let err = engine
            .ack(&reservation.id, &worker_id("w2"))
            .await
            .unwrap_err();
        assert!(matches!(err, EngineError::AccessDenied));
    }

    #[tokio::test]
    async fn double_ack_errors_the_second_time() {
        let engine = Engine::new();
        engine.create_queue(queue_id("q")).await.unwrap();
        engine.put(queue_id("q"), vec![1]).await.unwrap();
        let reservation = engine
            .reserve(queue_id("q"), worker_id("w1"))
            .await
            .unwrap()
            .unwrap();

        engine.ack(&reservation.id, &worker_id("w1")).await.unwrap();
        let err = engine
            .ack(&reservation.id, &worker_id("w1"))
            .await
            .unwrap_err();
        assert!(matches!(err, EngineError::ReservationNotFound(_)));
    }

    #[tokio::test]
    async fn concurrent_reserves_only_hand_out_the_job_once() {
        let engine = std::sync::Arc::new(Engine::new());
        engine.create_queue(queue_id("q")).await.unwrap();
        engine.put(queue_id("q"), vec![1]).await.unwrap();

        let e1 = engine.clone();
        let e2 = engine.clone();

        let (r1, r2) = tokio::join!(
            async move { e1.reserve(queue_id("q"), worker_id("w1")).await.unwrap() },
            async move { e2.reserve(queue_id("q"), worker_id("w2")).await.unwrap() },
        );

        let successes = [r1, r2].into_iter().filter(Option::is_some).count();
        assert_eq!(successes, 1);
    }
}
