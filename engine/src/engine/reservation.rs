use thiserror::Error;
use uuid::Uuid;

use crate::engine::JobID;

#[derive(Error, Debug)]
pub enum ReservationIDError {
    #[error("reservation ID cannot be empty")]
    Empty,
}

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

impl From<ReservationID> for String {
    fn from(id: ReservationID) -> String {
        id.0
    }
}

#[derive(Clone)]
pub struct Reservation {
    pub id: ReservationID,
    pub job_id: JobID,
    pub payload: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_empty() {
        // ReservationID doesn't derive Debug, so unwrap_err() can't be used here.
        let err = match ReservationID::new("") {
            Err(e) => e,
            Ok(_) => panic!("expected an error"),
        };
        assert!(matches!(err, ReservationIDError::Empty));
    }

    #[test]
    fn new_accepts_nonempty() {
        let id = ReservationID::new("res-1").unwrap();
        assert_eq!(id.as_str(), "res-1");
    }

    #[test]
    fn generate_produces_nonempty_id() {
        let id = ReservationID::generate();
        assert!(!id.as_str().is_empty());
    }

    #[test]
    fn generate_produces_unique_ids() {
        let a = ReservationID::generate();
        let b = ReservationID::generate();
        assert_ne!(a.as_str(), b.as_str());
    }
}
