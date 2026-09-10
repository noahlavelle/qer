use uuid::Uuid;

use crate::engine::QueueID;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct JobID(String);

impl JobID {
    pub fn generate() -> Self {
        Self(Uuid::now_v7().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<JobID> for String {
    fn from(id: JobID) -> String {
        id.0
    }
}

impl Default for JobID {
    fn default() -> Self {
        Self::generate()
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Job {
    pub id: JobID,
    pub queue_id: QueueID,
    pub payload: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_generates_nonempty_id() {
        let id = JobID::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn new_generates_unique_ids() {
        let a = JobID::generate();
        let b = JobID::generate();
        assert_ne!(a.as_str(), b.as_str());
    }

    #[test]
    fn default_generates_an_id() {
        let id = JobID::default();
        assert!(!id.as_str().is_empty());
    }
}
