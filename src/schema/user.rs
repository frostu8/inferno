//! User operations.

use sqlx::{Executor, PgPool, Postgres};

use chrono::Utc;

/// A full user record.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
}

/// A full password login record.
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct PasswordLogin {
    pub user_id: i32,
    pub password_hash: String,
    pub salt: String,
}

/// Gets information about a user by username.
pub async fn get_user<'c, E>(db: E, username: &str) -> Result<Option<User>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        r#"
        SELECT id, username
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(db)
    .await
}

/// Fetches the password login of a user.
pub async fn get_password_login<'c, E>(
    db: &PgPool,
    username: &str,
) -> Result<Option<PasswordLogin>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        r#"
        SELECT l.user_id, l.password_hash, l.salt
        FROM logins l
        JOIN users u ON l.user_id = u.id
        WHERE u.username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(db)
    .await
}

/// Creates a user account with no login, returning the user.
pub async fn create_user<'c, E>(db: &PgPool, username: &str) -> Result<User, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let inserted_at = Utc::now();

    sqlx::query_as(
        r#"
        INSERT INTO users (username, inserted_at, updated_at)
        VALUES ($1, $2, $2)
        RETURNING id, username
        "#,
    )
    .bind(username)
    .bind(inserted_at)
    .bind(inserted_at)
    .fetch_one(db)
    .await
}
