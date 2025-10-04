//! Exposes methods for interacting with an external database.
//!
//! Almost all of these methods require the database to be passed, along with
//! any parameters that might be needed.

pub mod page;
pub mod session;
pub mod user;

pub type Database = sqlx::Any;
