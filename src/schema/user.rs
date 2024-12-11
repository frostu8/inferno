//! User operations.

use sqlx::PgPool;

use chrono::Utc;

/// A user record.
#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
}

/// A password login record.
#[derive(sqlx::FromRow)]
pub struct PasswordLogin {
    pub user_id: i32,
    pub password_hash: String,
    pub salt: String,
}

/// Gets information about a user by username.
pub async fn get_user(db: &PgPool, username: &str) -> Result<Option<User>, sqlx::Error> {
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
pub async fn get_password_login(
    db: &PgPool,
    username: &str,
) -> Result<Option<PasswordLogin>, sqlx::Error> {
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

/// Creates a user account with no login, returning the ID of the user.
pub async fn create_user(db: &PgPool, username: &str) -> Result<i32, sqlx::Error> {
    let inserted_at = Utc::now();

    sqlx::query_as::<_, (i32,)>(
        r#"
        INSERT INTO users (username, inserted_at, updated_at)
        VALUES ($1, $2, $2)
        RETURNING id
        "#,
    )
    .bind(username)
    .bind(inserted_at)
    .bind(inserted_at)
    .fetch_one(db)
    .await
    .map(|(id,)| id)
}
