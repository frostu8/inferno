//! Page information.
//!
//! The use of [`Slug`] in this module gaurantees that no bad accesses can be
//! made.

use base16::encode_lower;
use chrono::Utc;
use sqlx::{Executor, Postgres};

use tracing::instrument;

use sha2::{Digest, Sha256};

use crate::slug::Slug;
use crate::universe::Locator;

/// Result of [`get_page_content`] and [`get_page_for_update`].
#[derive(sqlx::FromRow)]
pub struct Page {
    pub path: Slug,
    pub content: String,
    pub latest_change_hash: String,
}

/// Gets the content of a page.
#[instrument]
pub async fn get_page_content<'c, E>(
    db: E,
    Locator { universe_id, path }: Locator<'_>,
) -> Result<Option<Page>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        r#"
        SELECT p.path, p.content, c.hash AS latest_change_hash
        FROM pages p
        RIGHT JOIN changes c ON c.page_id = p.id
        WHERE
            path = $1 AND
            universe_id = $2
        ORDER BY c.inserted_at DESC
        LIMIT 1
        "#,
    )
    .bind(path.as_str())
    .bind(universe_id)
    .fetch_optional(db)
    .await
}

/// Gets the content of a page for an update.
///
/// This function sets up a lock for an update, as opposed to
/// [`get_page_content`]. If you just want the page, use [`get_page_content`].
#[instrument]
pub async fn get_page_content_for_update<'c, E>(
    db: E,
    Locator { universe_id, path }: Locator<'_>,
) -> Result<Option<Page>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as(
        r#"
        SELECT p.path, p.content, c.hash AS latest_change_hash
        FROM pages p
        RIGHT JOIN changes c ON c.page_id = p.id
        WHERE
            path = $1 AND
            universe_id = $2
        ORDER BY c.inserted_at DESC
        LIMIT 1
        FOR UPDATE
        "#,
    )
    .bind(path.as_str())
    .bind(universe_id)
    .fetch_optional(db)
    .await
}

/// Updates the page content. Inserts a new page if it did not exist.
///
/// This does not log the diff, breaking diff operations; this function should
/// typically be called in conjunction with [`save_change`].
#[instrument]
pub async fn update_page_content<'c, E>(
    db: E,
    Locator { universe_id, path }: Locator<'_>,
    content: &str,
) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    let updated_at = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO pages (universe_id, path, content, inserted_at, updated_at)
        VALUES ($4, $1, $2, $3, $3)
        ON CONFLICT (path, universe_id) DO UPDATE
        SET
            content = excluded.content,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(path.as_str())
    .bind(content)
    .bind(updated_at)
    .bind(universe_id)
    .execute(db)
    .await
    .map(|_| ())
}

/// Saves a new change to the database. Returns the change hash.
///
/// This does not actually modify the page; this function should typically be
/// called in conjunction with [`update_page_content`].
pub async fn save_change<'c, E>(
    db: E,
    Locator { universe_id, path }: Locator<'_>,
    author: &str,
    changes: &str,
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
            p.universe_id = $6 AND
            u.username = $2
        "#,
    )
    .bind(path.as_str())
    .bind(author)
    .bind(&hash)
    .bind(changes)
    .bind(inserted_at)
    .bind(universe_id)
    .execute(db)
    .await
    .map(|_| hash)
}

/// Gets all the links registered in the database from a page.
#[instrument]
pub async fn get_links_from<'c, E>(
    db: E,
    Locator { universe_id, path }: Locator<'_>,
) -> Result<Vec<Slug>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as::<_, (String,)>(
        r#"
        SELECT l.dest_path
        FROM pages p
        RIGHT JOIN links l ON p.id = l.source_id
        WHERE
            p.path = $1 AND
            (p.universe_id = $2 OR p.universe_id IS NULL AND $2 IS NULL)
        "#,
    )
    .bind(path.as_str())
    .bind(universe_id)
    .fetch_all(db)
    .await
    .map(|inner| {
        inner
            .into_iter()
            .filter_map(|(s,)| Slug::new(s).ok())
            .collect()
    })
}

/// Gets all the links registered in the database from a page, filtering only
/// the ones that exist
#[instrument]
pub async fn get_existing_links_from<'c, E>(
    db: E,
    Locator { universe_id, path }: Locator<'_>,
) -> Result<Vec<Slug>, sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query_as::<_, (String,)>(
        r#"
        SELECT l.dest_path
        FROM pages p
        RIGHT JOIN links l ON p.id = l.source_id
        JOIN pages p2 ON p2.path = l.dest_path
        WHERE
            p.path = $1 AND
            p.universe_id = $2
        "#,
    )
    .bind(path.as_str())
    .bind(universe_id)
    .fetch_all(db)
    .await
    .map(|inner| {
        inner
            .into_iter()
            .filter_map(|(s,)| Slug::new(s).ok())
            .collect()
    })
}

/// Adds a new relational link.
pub async fn establish_link<'c, E>(
    db: E,
    Locator {
        universe_id,
        path: from,
    }: Locator<'_>,
    to: &Slug,
) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query(
        r#"
        INSERT INTO links (source_id, dest_path)
        SELECT p.id, $2
        FROM pages p
        WHERE
            p.path = $1 AND
            p.universe_id = $3
        ON CONFLICT (source_id, dest_path)
        DO NOTHING
        "#,
    )
    .bind(from.as_str())
    .bind(to.as_str())
    .bind(universe_id)
    .execute(db)
    .await
    .map(|_| ())
}

/// Deletes a relational link.
#[instrument]
pub async fn deregister_link<'c, E>(
    db: E,
    Locator {
        universe_id,
        path: from,
    }: Locator<'_>,
    to: &Slug,
) -> Result<(), sqlx::Error>
where
    E: Executor<'c, Database = Postgres>,
{
    sqlx::query(
        r#"
        DELETE FROM links l
        USING (
            SELECT id
            FROM pages
            WHERE
                path = $1 AND
                universe_id = $3
        ) AS p
        WHERE
            l.source_id = p.id AND
            l.dest_path = $2
        "#,
    )
    .bind(from.as_str())
    .bind(to.as_str())
    .bind(universe_id)
    .execute(db)
    .await
    .map(|_| ())
}
