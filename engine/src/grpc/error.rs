use tonic::Status;

use crate::engine::EngineError;

pub fn to_status(error: EngineError) -> Status {
    match error {
        EngineError::QueueNotFound(name) => {
            Status::not_found(format!("queue not found: {name}"))
        }
        EngineError::QueueAlreadyExists(name) => {
            Status::already_exists(format!("queue already exists: {name}"))
        }
        EngineError::ReservationNotFound(name) => {
            Status::not_found(format!("reservation not found: {name}"))
        }
        _ => {
            Status::internal("unknown")
        }
    }
}
