//! General server-only types and functions.

use crate::{passwords, schema::user, server::ServerState};

use clap::{Args, Parser, Subcommand};

use sqlx::PgPool;

/// The Inferno wiki-management system.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    /// Parses the args from the environment.
    pub fn parse() -> Cli {
        <Cli as Parser>::parse()
    }

    /// Runs the command-line interface.
    ///
    /// May never return if a command was processed.
    pub async fn run(&self, state: &ServerState) {
        let Some(command) = self.command.as_ref() else {
            return;
        };

        match command {
            Command::Create(Create::User(cmd)) => {
                if let Err(err) =
                    create_user_with_password(&state.pool, &cmd.username, &cmd.password).await
                {
                    leptos::logging::log!("failed to create user: {}", err);
                    std::process::exit(1);
                }
            }
            Command::Create(Create::SigningKey) => {
                let key = crate::server::random_signing_key();
                print!("{}", key);
            }
        }

        std::process::exit(0);
    }
}

/// Performs an operation.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Creates a new object in the database.
    #[command(subcommand)]
    Create(Create),
}

/// Creates a new object in the database.
#[derive(Debug, Subcommand)]
pub enum Create {
    /// Creates a new user.
    User(CreateUser),
    /// Creates a new signing key for use in `TOKEN_SIGNING_KEY`
    SigningKey,
}

/// Creates a new object in the database.
#[derive(Debug, Args)]
pub struct CreateUser {
    /// The username of the account.
    #[arg(short = 'U', long)]
    pub username: String,
    /// The password of the account.
    #[arg(short, long)]
    pub password: String,
}

/// Creates a new user in the database manually.
pub async fn create_user_with_password(
    db: &PgPool,
    username: &str,
    password: &str,
) -> Result<(), sqlx::Error> {
    // generate login
    let salt = passwords::generate_salt(passwords::SALT_LENGTH);

    // hash password
    let hashed = passwords::hash_password(password, &salt);

    // create user account
    let id = user::create_user(db, username).await?;

    // create login
    sqlx::query(
        r#"
        INSERT INTO logins (user_id, password_hash, salt)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(id)
    .bind(hashed)
    .bind(salt)
    .execute(db)
    .await?;

    Ok(())
}
