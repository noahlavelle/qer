use std::collections::VecDeque;

use crate::engine::{Job, error::QueueIDError};

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
