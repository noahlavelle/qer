use tonic::Status;

use crate::{engine::{EngineError, QueueIDError, ReservationIDError, WorkerIDError}, grpc::auth::AuthError};

impl From<EngineError> for Status {
    fn from(error: EngineError) -> Self {
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
            EngineError::AccessDenied => {
                Status::permission_denied("access denied")
            }
            _ => {
                Status::internal("unknown")
            }
        }
    }
}

impl From<QueueIDError> for Status {
    fn from(error: QueueIDError) -> Self {
        Status::invalid_argument(error.to_string())
    }
}

impl From<ReservationIDError> for Status {
    fn from(error: ReservationIDError) -> Self {
        Status::invalid_argument(error.to_string())
    }
}

impl From<WorkerIDError> for Status {
    fn from(error: WorkerIDError) -> Self {
        Status::invalid_argument(error.to_string())
    }
}

impl From<AuthError> for Status {
    fn from(error: AuthError) -> Self {
        Status::unauthenticated(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use tonic::Code;

    use super::*;

    #[test]
    fn queue_not_found_maps_to_not_found() {
        let status: Status = EngineError::QueueNotFound("q".to_owned()).into();
        assert_eq!(status.code(), Code::NotFound);
    }

    #[test]
    fn queue_already_exists_maps_to_already_exists() {
        let status: Status = EngineError::QueueAlreadyExists("q".to_owned()).into();
        assert_eq!(status.code(), Code::AlreadyExists);
    }

    #[test]
    fn reservation_not_found_maps_to_not_found() {
        let status: Status = EngineError::ReservationNotFound("r".to_owned()).into();
        assert_eq!(status.code(), Code::NotFound);
    }

    #[test]
    fn access_denied_maps_to_permission_denied() {
        let status: Status = EngineError::AccessDenied.into();
        assert_eq!(status.code(), Code::PermissionDenied);
    }

    #[test]
    fn unmapped_engine_errors_fall_back_to_internal() {
        let status: Status = EngineError::QueueEmpty.into();
        assert_eq!(status.code(), Code::Internal);
    }

    #[test]
    fn queue_id_error_maps_to_invalid_argument() {
        let status: Status = QueueIDError::Empty.into();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[test]
    fn reservation_id_error_maps_to_invalid_argument() {
        let status: Status = ReservationIDError::Empty.into();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[test]
    fn worker_id_error_maps_to_invalid_argument() {
        let status: Status = WorkerIDError::Empty.into();
        assert_eq!(status.code(), Code::InvalidArgument);
    }

    #[test]
    fn auth_error_maps_to_unauthenticated() {
        let status: Status = AuthError::TokenExpired.into();
        assert_eq!(status.code(), Code::Unauthenticated);
    }

    #[test]
    fn not_scoped_maps_to_unauthenticated() {
        let status: Status = AuthError::NotScoped.into();
        assert_eq!(status.code(), Code::Unauthenticated);
    }
}
