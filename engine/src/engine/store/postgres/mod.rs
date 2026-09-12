use std::str::FromStr;

use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};

use crate::engine::StoreError;

pub mod payload;
mod records;

pub async fn connect(database_url: &str) -> Result<PgPool, StoreError> {
    let options = PgConnectOptions::from_str(database_url)?.ssl_mode(PgSslMode::Disable);

    let pool = PgPoolOptions::new()
        .min_connections(1)
        .test_before_acquire(false)
        .connect_with(options)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
