mod proto;

use proto::qr::v1::{
    CheckHealthRequest, CheckHealthResponse,
    queue_engine_server::{QueueEngine, QueueEngineServer},
};
use tonic::{Request, Response, Status, transport::Server};
use tonic_health::server::health_reporter;

#[derive(Default)]
struct QueueEngineService;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;

    let (health_reporter, health_service) = health_reporter();

    health_reporter
        .set_serving::<QueueEngineServer<QueueEngineService>>()
        .await;

    println!("Queue engine listening on {addr}");

    Server::builder()
        .add_service(health_service)
        .add_service(QueueEngineServer::new(QueueEngineService::default()))
        .serve(addr)
        .await?;

    Ok(())
}
