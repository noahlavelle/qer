use tonic::{Request, Response, Status};

use crate::{
    engine::{Engine, QueueID, ReservationID, WorkerID}, grpc::error::to_status, proto::qer::{self, v1::{
        AckRequest, AckResponse, CheckHealthRequest, CheckHealthResponse, CreateQueueRequest, CreateQueueResponse, PutRequest, PutResponse, ReserveRequest, ReserveResponse, queue_engine_server::QueueEngine,
    }},
};

pub struct QueueEngineService {
    engine: Engine,
}

impl QueueEngineService {
    pub fn new(engine: Engine) -> Self {
        Self {
            engine
        }
    }
}

#[tonic::async_trait]
impl QueueEngine for QueueEngineService {
    async fn check_health(
        &self,
        _request: Request<CheckHealthRequest>,
    ) -> Result<Response<CheckHealthResponse>, Status> {
        Ok(Response::new(CheckHealthResponse {
            status: "ok".to_string(),
        }))
    }

    async fn create_queue(
        &self,
        request: Request<CreateQueueRequest>
    ) -> Result<Response<CreateQueueResponse>, Status> {
        let request = request.into_inner();

        let queue_id = QueueID::new(request.name)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        self.engine
            .create_queue(queue_id)
            .await
            .map_err(to_status)?;

        Ok(Response::new(CreateQueueResponse {}))
    }

    async fn put(
        &self,
        request: Request<PutRequest>,
    ) -> Result<Response<PutResponse>, Status> {
        let request = request.into_inner();

        let queue_id = QueueID::new(request.queue_name)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let job_id = self.engine
            .put(&queue_id, request.payload)
            .await
            .map_err(to_status)?;

        Ok(Response::new(PutResponse {
            job_id: job_id.as_str().to_owned()
        }))
    }

    async fn reserve(
        &self,
        request: Request<ReserveRequest>,
    ) -> Result<Response<ReserveResponse>, Status> {
        let request = request.into_inner();

        let queue_id = QueueID::new(request.queue_name)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let worker_id = WorkerID::new(request.worker_token)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        let reservation = self.engine
            .reserve(&queue_id, &worker_id)
            .await
            .map_err(to_status)?;

        match reservation {
            Some(reservation) => Ok(Response::new(ReserveResponse {
                reservation: Some(qer::v1::Reservation {
                    reservation_id: reservation.id.as_str().to_owned(),
                    job_id: reservation.job.id.as_str().to_owned(),
                    payload: reservation.job.payload,
                })
            })),
            None => Ok(Response::new(ReserveResponse {
                reservation: None
            }))
        }
    }

    async fn ack(
        &self,
        request: Request<AckRequest>,
    ) -> Result<Response<AckResponse>, Status> {
        let request = request.into_inner();

        let reservation_id = ReservationID::new(request.reservation_id)
            .map_err(|err| Status::invalid_argument(err.to_string()))?;

        self.engine
            .ack(&reservation_id)
            .await
            .map_err(to_status)?;

        Ok(Response::new(AckResponse {}))
    }
}
