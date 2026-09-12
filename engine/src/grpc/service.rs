use tonic::{Request, Response, Status};

use crate::{
    engine::{Engine, QueueID, ReservationID},
    grpc::auth::{self},
    proto::qer::{
        self,
        v1::{
            AckRequest, AckResponse, CreateQueueRequest, CreateQueueResponse, PutRequest,
            PutResponse, ReserveRequest, ReserveResponse, Scope, queue_engine_server::QueueEngine,
        },
    },
};

pub struct QueueEngineService {
    engine: Engine,
}

impl QueueEngineService {
    pub fn new(engine: Engine) -> Self {
        Self { engine }
    }
}

#[tonic::async_trait]
impl QueueEngine for QueueEngineService {
    async fn create_queue(
        &self,
        request: Request<CreateQueueRequest>,
    ) -> Result<Response<CreateQueueResponse>, Status> {
        let authed_worker = auth::auth_from_request(&request)?;
        authed_worker.check_scope(Scope::QueueCreate)?;

        let request = request.into_inner();
        let queue_id = QueueID::new(request.name)?;

        self.engine.create_queue(queue_id).await?;

        Ok(Response::new(CreateQueueResponse {}))
    }

    async fn put(&self, request: Request<PutRequest>) -> Result<Response<PutResponse>, Status> {
        let authed_worker = auth::auth_from_request(&request)?;
        authed_worker.check_scope(Scope::QueueProduce)?;

        let request = request.into_inner();
        let queue_id = QueueID::new(request.queue_name)?;

        let job_id = self.engine.put(queue_id, request.payload).await?;

        Ok(Response::new(PutResponse {
            job_id: job_id.into(),
        }))
    }

    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> Result<Response<ReserveResponse>, Status> {
        let authed_worker = auth::auth_from_request(&request)?;
        authed_worker.check_scope(Scope::QueueConsume)?;

        let request = request.into_inner();
        let queue_id = QueueID::new(request.queue_name)?;

        let reservation = self
            .engine
            .reserve(&queue_id, authed_worker.worker_id)
            .await?;

        match reservation {
            Some(reservation) => Ok(Response::new(ReserveResponse {
                reservation: Some(qer::v1::Reservation {
                    reservation_id: reservation.id.into(),
                    job_id: reservation.job_id.into(),
                    payload: reservation.payload,
                }),
            })),
            None => Ok(Response::new(ReserveResponse { reservation: None })),
        }
    }

    async fn ack(&self, request: Request<AckRequest>) -> Result<Response<AckResponse>, Status> {
        let authed_worker = auth::auth_from_request(&request)?;
        authed_worker.check_scope(Scope::QueueConsume)?;

        let request = request.into_inner();
        let reservation_id = ReservationID::new(request.reservation_id)?;

        self.engine
            .ack(&reservation_id, &authed_worker.worker_id)
            .await?;

        Ok(Response::new(AckResponse {}))
    }
}

#[cfg(test)]
mod tests {
    use tonic::Code;

    use super::*;
    use crate::engine::{WorkerID, test_engine};

    async fn service() -> QueueEngineService {
        QueueEngineService::new(test_engine().await)
    }

    fn authed_request<T>(message: T, worker: &str, scopes: Vec<Scope>) -> Request<T> {
        let mut request = Request::new(message);
        request.extensions_mut().insert(auth::AuthenticatedWorker {
            worker_id: WorkerID::new(worker).unwrap(),
            scopes,
        });
        request
    }

    /// The generated proto message types don't derive Debug, so `unwrap_err()`
    /// (which requires the `Ok` variant to be `Debug`) can't be used on RPC results.
    fn expect_err<T, E>(result: Result<T, E>) -> E {
        match result {
            Err(e) => e,
            Ok(_) => panic!("expected an error"),
        }
    }

    #[tokio::test]
    async fn create_queue_succeeds() {
        let svc = service().await;
        let response = svc
            .create_queue(authed_request(
                CreateQueueRequest {
                    name: "orders".into(),
                },
                "worker-1",
                vec![Scope::QueueCreate],
            ))
            .await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn create_queue_without_auth_is_unauthenticated() {
        let svc = service().await;
        let result = svc
            .create_queue(Request::new(CreateQueueRequest {
                name: "orders".into(),
            }))
            .await;
        assert_eq!(expect_err(result).code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn create_queue_rejects_insufficient_scope() {
        let svc = service().await;
        let result = svc
            .create_queue(authed_request(
                CreateQueueRequest {
                    name: "orders".into(),
                },
                "worker-1",
                vec![Scope::QueueProduce],
            ))
            .await;
        assert_eq!(expect_err(result).code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn create_queue_rejects_empty_name() {
        let svc = service().await;
        let result = svc
            .create_queue(authed_request(
                CreateQueueRequest {
                    name: String::new(),
                },
                "worker-1",
                vec![Scope::QueueCreate],
            ))
            .await;
        assert_eq!(expect_err(result).code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn create_queue_rejects_duplicates() {
        let svc = service().await;
        svc.create_queue(authed_request(
            CreateQueueRequest {
                name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueCreate],
        ))
        .await
        .unwrap();

        let result = svc
            .create_queue(authed_request(
                CreateQueueRequest {
                    name: "orders".into(),
                },
                "worker-1",
                vec![Scope::QueueCreate],
            ))
            .await;
        assert_eq!(expect_err(result).code(), Code::AlreadyExists);
    }

    #[tokio::test]
    async fn put_succeeds_and_returns_a_job_id() {
        let svc = service().await;
        svc.create_queue(authed_request(
            CreateQueueRequest {
                name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueCreate],
        ))
        .await
        .unwrap();

        let response = svc
            .put(authed_request(
                PutRequest {
                    queue_name: "orders".into(),
                    payload: vec![1, 2, 3],
                },
                "worker-1",
                vec![Scope::QueueProduce],
            ))
            .await
            .unwrap()
            .into_inner();

        assert!(!response.job_id.is_empty());
    }

    #[tokio::test]
    async fn put_without_auth_is_unauthenticated() {
        let svc = service().await;
        let result = svc
            .put(Request::new(PutRequest {
                queue_name: "missing".into(),
                payload: vec![],
            }))
            .await;
        assert_eq!(expect_err(result).code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn put_rejects_insufficient_scope() {
        let svc = service().await;
        let result = svc
            .put(authed_request(
                PutRequest {
                    queue_name: "missing".into(),
                    payload: vec![],
                },
                "worker-1",
                vec![Scope::QueueConsume],
            ))
            .await;
        assert_eq!(expect_err(result).code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn put_into_unknown_queue_is_not_found() {
        let svc = service().await;
        let result = svc
            .put(authed_request(
                PutRequest {
                    queue_name: "missing".into(),
                    payload: vec![],
                },
                "worker-1",
                vec![Scope::QueueProduce],
            ))
            .await;
        assert_eq!(expect_err(result).code(), Code::NotFound);
    }

    #[tokio::test]
    async fn reserve_without_auth_is_unauthenticated() {
        let svc = service().await;
        svc.create_queue(authed_request(
            CreateQueueRequest {
                name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueCreate],
        ))
        .await
        .unwrap();

        let result = svc
            .reserve(Request::new(ReserveRequest {
                queue_name: "orders".into(),
            }))
            .await;
        assert_eq!(expect_err(result).code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn reserve_rejects_insufficient_scope() {
        let svc = service().await;
        svc.create_queue(authed_request(
            CreateQueueRequest {
                name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueCreate],
        ))
        .await
        .unwrap();

        let request = authed_request(
            ReserveRequest {
                queue_name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueProduce],
        );
        let result = svc.reserve(request).await;
        assert_eq!(expect_err(result).code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn reserve_returns_none_when_queue_is_empty() {
        let svc = service().await;
        svc.create_queue(authed_request(
            CreateQueueRequest {
                name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueCreate],
        ))
        .await
        .unwrap();

        let request = authed_request(
            ReserveRequest {
                queue_name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueConsume],
        );
        let response = svc.reserve(request).await.unwrap().into_inner();
        assert!(response.reservation.is_none());
    }

    #[tokio::test]
    async fn reserve_returns_a_job_when_present() {
        let svc = service().await;
        svc.create_queue(authed_request(
            CreateQueueRequest {
                name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueCreate],
        ))
        .await
        .unwrap();
        svc.put(authed_request(
            PutRequest {
                queue_name: "orders".into(),
                payload: b"hello".to_vec(),
            },
            "worker-1",
            vec![Scope::QueueProduce],
        ))
        .await
        .unwrap();

        let request = authed_request(
            ReserveRequest {
                queue_name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueConsume],
        );
        let response = svc.reserve(request).await.unwrap().into_inner();

        let reservation = response.reservation.expect("expected a reservation");
        assert_eq!(reservation.payload, b"hello");
        assert!(!reservation.reservation_id.is_empty());
    }

    #[tokio::test]
    async fn reserve_from_unknown_queue_is_not_found() {
        let svc = service().await;
        let request = authed_request(
            ReserveRequest {
                queue_name: "missing".into(),
            },
            "worker-1",
            vec![Scope::QueueConsume],
        );
        let result = svc.reserve(request).await;
        assert_eq!(expect_err(result).code(), Code::NotFound);
    }

    #[tokio::test]
    async fn ack_without_auth_is_unauthenticated() {
        let svc = service().await;
        let result = svc
            .ack(Request::new(AckRequest {
                reservation_id: "r1".into(),
            }))
            .await;
        assert_eq!(expect_err(result).code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn ack_rejects_insufficient_scope() {
        let svc = service().await;
        let request = authed_request(
            AckRequest {
                reservation_id: "r1".into(),
            },
            "worker-1",
            vec![Scope::QueueProduce],
        );
        let result = svc.ack(request).await;
        assert_eq!(expect_err(result).code(), Code::Unauthenticated);
    }

    #[tokio::test]
    async fn ack_succeeds_for_the_reserving_worker() {
        let svc = service().await;
        svc.create_queue(authed_request(
            CreateQueueRequest {
                name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueCreate],
        ))
        .await
        .unwrap();
        svc.put(authed_request(
            PutRequest {
                queue_name: "orders".into(),
                payload: vec![1],
            },
            "worker-1",
            vec![Scope::QueueProduce],
        ))
        .await
        .unwrap();

        let reserve_request = authed_request(
            ReserveRequest {
                queue_name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueConsume],
        );
        let reservation = svc
            .reserve(reserve_request)
            .await
            .unwrap()
            .into_inner()
            .reservation
            .unwrap();

        let ack_request = authed_request(
            AckRequest {
                reservation_id: reservation.reservation_id,
            },
            "worker-1",
            vec![Scope::QueueConsume],
        );
        let response = svc.ack(ack_request).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn ack_by_a_different_worker_is_rejected() {
        let svc = service().await;
        svc.create_queue(authed_request(
            CreateQueueRequest {
                name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueCreate],
        ))
        .await
        .unwrap();
        svc.put(authed_request(
            PutRequest {
                queue_name: "orders".into(),
                payload: vec![1],
            },
            "worker-1",
            vec![Scope::QueueProduce],
        ))
        .await
        .unwrap();

        let reserve_request = authed_request(
            ReserveRequest {
                queue_name: "orders".into(),
            },
            "worker-1",
            vec![Scope::QueueConsume],
        );
        let reservation = svc
            .reserve(reserve_request)
            .await
            .unwrap()
            .into_inner()
            .reservation
            .unwrap();

        let ack_request = authed_request(
            AckRequest {
                reservation_id: reservation.reservation_id,
            },
            "worker-2",
            vec![Scope::QueueConsume],
        );
        let result = svc.ack(ack_request).await;
        assert_eq!(expect_err(result).code(), Code::PermissionDenied);
    }

    #[tokio::test]
    async fn ack_unknown_reservation_is_not_found() {
        let svc = service().await;
        let request = authed_request(
            AckRequest {
                reservation_id: "missing".into(),
            },
            "worker-1",
            vec![Scope::QueueConsume],
        );
        let result = svc.ack(request).await;
        assert_eq!(expect_err(result).code(), Code::NotFound);
    }
}
