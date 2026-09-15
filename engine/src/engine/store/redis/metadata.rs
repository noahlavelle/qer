use redisclient::{AsyncCommands, aio::MultiplexedConnection};
use tonic::async_trait;

use crate::engine::{
    JobID, QueueID, ReservationID, StoreError, WorkerID,
    store::{MetadataStore, redis::scripts},
};

fn queues_key() -> &'static str {
    "qer:queues"
}

fn queue_jobs_key(queue_id: &QueueID) -> String {
    format!("qer:queue:{}:jobs", queue_id.as_str())
}

fn reservation_key(reservation_id: &ReservationID) -> String {
    format!("qer:reservation:{}", reservation_id.as_str())
}

pub struct RedisMetadataStore {
    conn: MultiplexedConnection,
}

impl RedisMetadataStore {
    pub fn new(conn: MultiplexedConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl MetadataStore for RedisMetadataStore {
    async fn create_queue(&self, queue_id: QueueID) -> Result<(), StoreError> {
        let mut conn = self.conn.clone();

        let added: bool = conn.sadd(queues_key(), queue_id.as_str()).await?;
        if !added {
            return Err(StoreError::QueueAlreadyExists(queue_id.as_str().to_owned()));
        }

        Ok(())
    }

    async fn enqueue(&self, queue_id: &QueueID, job_id: JobID) -> Result<(), StoreError> {
        let mut conn = self.conn.clone();

        let exists: bool = conn.sismember(queues_key(), queue_id.as_str()).await?;
        if !exists {
            return Err(StoreError::QueueNotFound(queue_id.as_str().to_owned()));
        }

        let _: () = conn
            .lpush(queue_jobs_key(queue_id), job_id.as_str())
            .await?;

        Ok(())
    }

    async fn reserve(
        &self,
        queue_id: &QueueID,
        reservation_id: ReservationID,
        worker_id: WorkerID,
    ) -> Result<Option<JobID>, StoreError> {
        let mut conn = self.conn.clone();

        let exists: bool = conn.sismember(queues_key(), queue_id.as_str()).await?;
        if !exists {
            return Err(StoreError::QueueNotFound(queue_id.as_str().to_owned()));
        }

        let job_id: Option<String> = scripts::RESERVE
            .key(queue_jobs_key(queue_id))
            .key(reservation_key(&reservation_id))
            .arg(worker_id.as_str())
            .invoke_async(&mut conn)
            .await?;

        Ok(job_id.map(JobID::new))
    }

    async fn ack(
        &self,
        reservation_id: &ReservationID,
        worker_id: &WorkerID,
    ) -> Result<JobID, StoreError> {
        let mut conn = self.conn.clone();

        let reply: Vec<String> = scripts::ACK
            .key(reservation_key(reservation_id))
            .arg(worker_id.as_str())
            .invoke_async(&mut conn)
            .await?;

        match reply.as_slice() {
            [status, job_id] if status == "ok" => Ok(JobID::new(job_id.into())),
            [status] if status == "denied" => Err(StoreError::AccessDenied),
            _ => Err(StoreError::ReservationNotFound(
                reservation_id.as_str().to_owned(),
            )),
        }
    }
}
