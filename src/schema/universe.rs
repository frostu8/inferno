//! Universe schema.

use sqlx::{Executor, Postgres};

use tracing::instrument;

use crate::universe::Universe;

#[derive(Debug)]
pub struct CreateUniverse<'a> {
    pub host: Option<&'a str>,
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
pub async fn create_universe<'c, E>(
    db: E,
    create: CreateUniverse<'_>,
) -> Result<Universe, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let CreateUniverse { host } = create;

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

/// Gets the global universe.
#[instrument]
pub async fn get_global_universe<'c, E>(db: E) -> Result<Universe, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as("SELECT id, host FROM universes WHERE id = 0")
        .fetch_one(db)
        .await
}

/// Creates the global universe.
#[instrument]
pub async fn create_global_universe<'c, E>(
    db: E,
    create: CreateUniverse<'_>,
) -> Result<Universe, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let CreateUniverse { host } = create;

    sqlx::query_as(
        r#"
        INSERT INTO universes (id, host)
        VALUES (0, $1)
        RETURNING id, host
        "#,
    )
    .bind(host)
    .fetch_one(db)
    .await
}
