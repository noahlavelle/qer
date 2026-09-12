use tonic::transport::Server;
use tonic_health::server::health_reporter;

use crate::{
    engine::Engine,
    grpc::{middleware, service::QueueEngineService},
    proto::qer::v1::queue_engine_server::QueueEngineServer,
};

mod engine;
mod grpc;
mod proto;

// TODO:
//   - Start to look at other resolved states
//     - Retry returning to the queue
//     - A manager to bury jobs over x retries etc

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;

    let engine = Engine::new();
    let service = QueueEngineService::new(engine);

    let (health_reporter, health_service) = health_reporter();

    health_reporter
        .set_serving::<QueueEngineServer<QueueEngineService>>()
        .await;

    println!("Queue engine listening on {addr}");

    Server::builder()
        .add_service(health_service)
        .add_service(QueueEngineServer::with_interceptor(
            service,
            middleware::auth_interceptor,
        ))
        .serve(addr)
        .await?;

    Ok(())
}
