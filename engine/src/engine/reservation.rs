use uuid::Uuid;

use crate::engine::{Job, WorkerID, error::ReservationIDError};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ReservationID(String);

impl ReservationID {
    pub fn new(value: impl Into<String>) -> Result<Self, ReservationIDError> {
        let value = value.into();

        if value.is_empty() {
            return Err(ReservationIDError::Empty);
        }

        Ok(Self(value))
    }

    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone)]
pub struct Reservation {
    pub id: ReservationID,
    pub job: Job,
    pub worker_id: WorkerID,
}
