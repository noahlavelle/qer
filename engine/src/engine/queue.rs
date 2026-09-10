use std::collections::VecDeque;

use thiserror::Error;

use crate::engine::Job;

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

pub struct Queue {
    ready: VecDeque<Job>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            ready: VecDeque::new(),
        }
    }

    pub fn put(&mut self, job: Job) {
        self.ready.push_back(job);
    }

    pub fn reserve(&mut self) -> Option<Job> {
        self.ready.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::JobID;

    fn make_job(queue_id: &QueueID, payload: &[u8]) -> Job {
        Job {
            id: JobID::generate(),
            queue_id: queue_id.clone(),
            payload: payload.to_vec(),
        }
    }

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

    #[test]
    fn reserve_on_empty_queue_returns_none() {
        let mut queue = Queue::new();
        assert!(queue.reserve().is_none());
    }

    #[test]
    fn put_then_reserve_returns_the_job() {
        let queue_id = QueueID::new("q").unwrap();
        let mut queue = Queue::new();
        let job = make_job(&queue_id, b"payload");
        let job_id = job.id.clone();

        queue.put(job);
        let reserved = queue.reserve().expect("expected a job");

        assert_eq!(reserved.id.as_str(), job_id.as_str());
        assert_eq!(reserved.payload, b"payload");
    }

    #[test]
    fn reserve_drains_queue_to_empty() {
        let queue_id = QueueID::new("q").unwrap();
        let mut queue = Queue::new();
        queue.put(make_job(&queue_id, b"a"));

        assert!(queue.reserve().is_some());
        assert!(queue.reserve().is_none());
    }

    #[test]
    fn multiple_jobs_are_reserved_in_fifo_order() {
        let queue_id = QueueID::new("q").unwrap();
        let mut queue = Queue::new();
        queue.put(make_job(&queue_id, b"first"));
        queue.put(make_job(&queue_id, b"second"));
        queue.put(make_job(&queue_id, b"third"));

        assert_eq!(queue.reserve().unwrap().payload, b"first");
        assert_eq!(queue.reserve().unwrap().payload, b"second");
        assert_eq!(queue.reserve().unwrap().payload, b"third");
        assert!(queue.reserve().is_none());
    }
}
