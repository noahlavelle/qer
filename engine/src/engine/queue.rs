use std::collections::VecDeque;

use thiserror::Error;

use crate::engine::JobID;

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
    ready: VecDeque<JobID>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            ready: VecDeque::new(),
        }
    }

    pub fn put(&mut self, job_id: JobID) {
        self.ready.push_back(job_id);
    }

    pub fn reserve(&mut self) -> Option<JobID> {
        self.ready.pop_front()
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

    #[test]
    fn reserve_on_empty_queue_returns_none() {
        let mut queue = Queue::new();
        assert!(queue.reserve().is_none());
    }

    #[test]
    fn put_then_reserve_returns_the_job() {
        let mut queue = Queue::new();
        let job_id = JobID::generate();

        queue.put(job_id.clone());
        let reserved = queue.reserve().expect("expected a job");

        assert_eq!(reserved.as_str(), job_id.as_str());
    }

    #[test]
    fn reserve_drains_queue_to_empty() {
        let mut queue = Queue::new();
        queue.put(JobID::generate());

        assert!(queue.reserve().is_some());
        assert!(queue.reserve().is_none());
    }

    #[test]
    fn multiple_jobs_are_reserved_in_fifo_order() {
        let mut queue = Queue::new();
        let first = JobID::generate();
        let second = JobID::generate();
        let third = JobID::generate();

        queue.put(first.clone());
        queue.put(second.clone());
        queue.put(third.clone());

        assert_eq!(queue.reserve().unwrap().as_str(), first.as_str());
        assert_eq!(queue.reserve().unwrap().as_str(), second.as_str());
        assert_eq!(queue.reserve().unwrap().as_str(), third.as_str());
        assert!(queue.reserve().is_none());
    }
}
