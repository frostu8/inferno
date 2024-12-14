//! Page information.
//!
//! The use of [`Slug`] in this module gaurantees that no bad accesses can be
//! made.

use base16::encode_lower;
use chrono::Utc;
use sqlx::{Executor, Postgres};

use sha2::{Digest, Sha256};

use crate::slug::Slug;

/// Result of [`get_page_content`] and [`get_page_for_update`].
#[derive(sqlx::FromRow)]
pub struct Page {
    pub content: String,
    pub latest_change_hash: String,
}

/// Gets the content of a page.
pub async fn get_page_content<'c, E>(path: &Slug, db: E) -> Result<Option<Page>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        r#"
        SELECT p.content, c.hash AS latest_change_hash
        FROM pages p
        RIGHT JOIN changes c ON c.page_id = p.id
        WHERE path = $1
        ORDER BY c.inserted_at DESC
        LIMIT 1
        "#,
    )
    .bind(path.as_str())
    .fetch_optional(db)
    .await
}

/// Gets the content of a page for an update.
///
/// This function sets up a lock for an update, as opposed to
/// [`get_page_content`]. If you just want the page, use [`get_page_content`].
pub async fn get_page_content_for_update<'c, E>(
    path: &Slug,
    db: E,
) -> Result<Option<Page>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        r#"
        SELECT p.content, c.hash AS latest_change_hash
        FROM pages p
        RIGHT JOIN changes c ON c.page_id = p.id
        WHERE path = $1
        ORDER BY c.inserted_at DESC
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(path.as_str())
    .fetch_optional(db)
    .await
}

/// Updates the page content. Inserts a new page if it did not exist.
///
/// This does not log the diff, breaking diff operations; this function should
/// typically be called in conjunction with [`save_change`].
pub async fn update_page_content<'c, E>(
    path: &Slug,
    content: &str,
    db: E,
) -> Result<(), sqlx::Error>
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
    .bind(path.as_str())
    .bind(content)
    .bind(updated_at)
    .execute(db)
    .await
    .map(|_| ())
}

/// Saves a new change to the database. Returns the change hash.
///
/// This does not actually modify the page; this function should typically be
/// called in conjunction with [`update_page_content`].
pub async fn save_change<'c, E>(
    path: &Slug,
    author: &str,
    changes: &str,
    db: E,
) -> Result<String, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let inserted_at = Utc::now();

    // calc hash
    let mut hasher = Sha256::new();

    // also has the author, page path and time
    hasher.update(path.as_str());
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
    .bind(path.as_str())
    .bind(author)
    .bind(&hash)
    .bind(changes)
    .bind(inserted_at)
    .execute(db)
    .await
    .map(|_| hash)
}
