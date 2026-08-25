use uuid::Uuid;

use crate::engine::queue::QueueID;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct JobID(String);

impl JobID {
    pub fn new() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone)]
pub struct Job {
    pub id: JobID,
    pub queue_id: QueueID,
    pub payload: Vec<u8>,
}
