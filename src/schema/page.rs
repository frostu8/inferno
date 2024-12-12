//! Page information.

use base16::encode_lower;
use chrono::Utc;
use sqlx::{Executor, PgPool, Postgres};

use sha2::{Digest, Sha256};

/// Gets the content of a page, returning it as a [`String`].
pub async fn get_page_content(path: &str, db: &PgPool) -> Result<Option<String>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct Page {
        pub content: String,
    }

    // get page
    sqlx::query_as::<_, Page>("SELECT content FROM pages WHERE path = $1")
        .bind(path)
        .fetch_optional(db)
        .await
        .map(|result| result.map(|Page { content }| content))
}

/// Updates the page content. Inserts a new page if it did not exist.
///
/// This does not log the diff, breaking diff operations; this function should
/// typically be called in conjunction with [`save_change`].
pub async fn update_page_content<'c, E>(path: &str, content: &str, db: E) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let updated_at = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO pages (path, content, inserted_at, updated_at)
        VALUES ($1, $2, $3, $3)
        ON CONFLICT (path) DO UPDATE
        SET
            content = excluded.content,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(path)
    .bind(content)
    .bind(updated_at)
    .execute(db)
    .await
    .map(|_| ())
}

/// Saves a new change to the database.
///
/// This does not actually modify the page; this function should typically be
/// called in conjunction with [`update_page_content`].
pub async fn save_change<'c, E>(
    path: &str,
    author: &str,
    changes: &str,
    db: E,
) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let inserted_at = Utc::now();

    // calc hash
    let mut hasher = Sha256::new();

    // also has the author, page path and time
    hasher.update(path);
    hasher.update(author);
    hasher.update(inserted_at.timestamp().to_le_bytes());
    // hash changes
    hasher.update(changes);

    let hash = hasher.finalize();
    let hash = encode_lower(&hash);

    // save change
    sqlx::query(
        r#"
        INSERT INTO changes (page_id, author_id, hash, content, inserted_at)
        SELECT p.id, u.id, $3, $4, $5
        FROM pages p, users u
        WHERE
            p.path = $1 AND
            u.username = $2
        "#,
    )
    .bind(path)
    .bind(author)
    .bind(&hash)
    .bind(changes)
    .bind(inserted_at)
    .execute(db)
    .await
    .map(|_| ())
}
