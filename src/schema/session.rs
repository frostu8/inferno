//! Session storage.

use chrono::Utc;

use sqlx::Executor;

use super::Database as PreferredDatabase;

#[derive(Debug, sqlx::FromRow)]
pub struct SessionUser {
    pub session_id: i32,
    pub id: i32,
    pub username: String,
}

/// Creates a new session.
pub async fn create_session<'c, E>(db: E, username: &str, hash: &str) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = PreferredDatabase>,
{
    assert!(hash.len() <= 64);

    let inserted_at = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO sessions (user_id, hash, inserted_at, updated_at)
        SELECT id, $2, $3, $3
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(username)
    .bind(hash)
    .bind(format!("{}", inserted_at.format("%+")))
    .execute(db)
    .await
    .map(|_| ())
}

/// Fetches a session by hash.
pub async fn get_session<'c, E>(db: E, hash: &str) -> Result<Option<SessionUser>, sqlx::Error>
where
    E: Executor<'c, Database = PreferredDatabase>,
{
    assert!(hash.len() <= 64);

    sqlx::query_as(
        r#"
        SELECT s.id AS session_id, u.id, u.username
        FROM sessions s, users u
        WHERE
            s.user_id = u.id AND
            s.hash = $1 AND
            NOT s.disposed
        "#,
    )
    .bind(hash)
    .fetch_optional(db)
    .await
}

/// Destroys a session by hash.
pub async fn dispose_session<'c, E>(db: E, hash: &str) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = PreferredDatabase>,
{
    let updated_at = Utc::now();

    sqlx::query(
        r#"
        UPDATE sessions
        SET
            disposed = true,
            updated_at = $2
        WHERE
            hash = $1 AND
            NOT disposed
        "#,
    )
    .bind(hash)
    .bind(format!("{}", updated_at.format("%+")))
    .execute(db)
    .await
    .map(|_| ())
}
