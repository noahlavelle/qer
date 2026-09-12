use sqlx::{PgPool, types::Uuid};
use tonic::async_trait;

use crate::engine::{JobID, StoreError, store::PayloadStore};

pub struct PostgresPayloadStore {
    pool: PgPool,
}

impl PostgresPayloadStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// `JobID` is only ever constructed via `JobID::generate()` (a `Uuid::now_v7()` string),
/// so this can't realistically fail - there's no `JobID::new` that accepts arbitrary
/// strings. The `job_payloads.job_id` column is `UUID`, and Postgres won't implicitly
/// cast a bound text parameter to it, so this parse is required, not just a nicety.
fn job_uuid(job_id: &JobID) -> Uuid {
    Uuid::parse_str(job_id.as_str()).expect("JobID is always a valid UUID")
}

#[async_trait]
impl PayloadStore for PostgresPayloadStore {
    async fn put(&self, job_id: JobID, payload: Vec<u8>) -> Result<(), StoreError> {
        sqlx::query("INSERT INTO job_payloads (job_id, payload) VALUES ($1, $2)")
            .bind(job_uuid(&job_id))
            .bind(payload)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get(&self, job_id: &JobID) -> Result<Vec<u8>, StoreError> {
        sqlx::query_scalar::<_, Vec<u8>>("SELECT payload FROM job_payloads WHERE job_id = $1")
            .bind(job_uuid(job_id))
            .fetch_one(&self.pool)
            .await
            .map_err(|err| match err {
                sqlx::Error::RowNotFound => StoreError::JobNotFound(job_id.as_str().to_owned()),
                err => StoreError::Postgres(err),
            })
    }

    async fn delete(&self, job_id: &JobID) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM job_payloads WHERE job_id = $1")
            .bind(job_uuid(job_id))
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
