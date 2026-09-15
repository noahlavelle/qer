use std::time::Duration;

use tokio::time::sleep;
use tonic::transport::Server;
use tonic_health::server::health_reporter;

use crate::{
    engine::{
        Engine, StoreError,
        store::{postgres, redis},
    },
    grpc::{QueueEngineService, middleware},
    proto::qer::v1::queue_engine_server::QueueEngineServer,
};

mod engine;
mod grpc;
mod proto;

// TODO:
//   - Add an in memory cache for the postgres layer, toggleable
//   - Start to look at other resolved states
//     - Retry returning to the queue
//     - A manager to bury jobs over x retries etc

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;

    let redis_client = connect_with_retry(
        || redis::connect("redis://redis:6379"),
        5,
        Duration::from_secs(2),
    )
    .await?;
    let postgres_connection = connect_with_retry(
        || postgres::connect("postgres://qer:dev@postgres:5432/qer"),
        5,
        Duration::from_secs(2),
    )
    .await?;
    let engine = Engine::new(redis_client, postgres_connection);
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

async fn connect_with_retry<F, Fut, T>(
    mut connect: F,
    attempts: u32,
    delay: Duration,
) -> Result<T, StoreError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, StoreError>>,
{
    for attempt in 1..attempts {
        match connect().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                eprint!(
                    "connect attempt {attempt}/{attempts} failed: {err}; retrying in {delay:?}"
                );
                sleep(delay).await;
            }
        }
    }

    connect().await
}
