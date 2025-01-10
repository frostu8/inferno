//! Universe schema.

use sqlx::{Executor, Postgres};

use tracing::instrument;

#[derive(sqlx::FromRow, Debug)]
pub struct Universe {
    pub id: i32,
    pub host: String,
}

/// Fetches a universe by host.
#[instrument]
pub async fn get_universe_by_host<'c, E>(db: E, host: &str) -> Result<Option<Universe>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        r#"
        SELECT id, host
        FROM universes
        WHERE host = $1
        "#,
    )
    .bind(host)
    .fetch_optional(db)
    .await
}

/// Creates a new universe.
#[instrument]
pub async fn create_universe<'c, E>(db: E, host: &str) -> Result<Universe, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        r#"
        INSERT INTO universes (host)
        VALUES ($1)
        RETURNING id, host
        "#,
    )
    .bind(host)
    .fetch_one(db)
    .await
}
