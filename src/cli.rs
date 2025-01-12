//! General server-only types and functions.

use crate::{
    crypto,
    schema::{
        universe::{create_universe, CreateUniverse as CreateUniverseSchema},
        user,
    },
    ServerState,
};

use clap::{Args, Parser, Subcommand};

use eyre::{Report, WrapErr};

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
    pub async fn run(&self, state: &ServerState) -> Result<ShouldContinue, Report> {
        if let Some(command) = self.command.as_ref() {
            match command {
                Command::Create(Create::User(cmd)) => {
                    create_user_with_password(&state.pool, &cmd.username, &cmd.password)
                        .await
                        .wrap_err("failed to create user")?;
                }
                Command::Create(Create::Universe(cmd)) => {
                    create_universe(
                        &state.pool,
                        CreateUniverseSchema {
                            host: Some(&cmd.host),
                        },
                    )
                    .await
                    .wrap_err("failed to create universe")?;
                }
                Command::Create(Create::SigningKey) => {
                    let key = crate::random_signing_key();
                    print!("{}", key);
                }
            }

            Ok(ShouldContinue::Exit)
        } else {
            Ok(ShouldContinue::Daemon)
        }
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
    /// Creates a new universe.
    Universe(CreateUniverse),
    /// Creates a new signing key for use in `TOKEN_SIGNING_KEY`
    SigningKey,
}

/// Creates a new user in the database.
#[derive(Debug, Args)]
pub struct CreateUser {
    /// The username of the account.
    #[arg(short = 'U', long)]
    pub username: String,
    /// The password of the account.
    #[arg(short, long)]
    pub password: String,
}

/// Creates a new universe  in the database.
#[derive(Debug, Args)]
pub struct CreateUniverse {
    /// The virtual host of the universe.
    #[arg(short = 'H', long)]
    pub host: String,
}

pub enum ShouldContinue {
    /// The server should start as normal.
    Daemon,
    /// A one-time command was issued and the process should exit.
    Exit,
}

/// Creates a new user in the database manually.
pub async fn create_user_with_password(
    db: &PgPool,
    username: &str,
    password: &str,
) -> Result<(), sqlx::Error> {
    // generate login
    let salt = crypto::generate_salt(crypto::SALT_LENGTH);

    // hash password
    let hashed = crypto::hash_password(password, &salt);

    // create user account
    let user = user::create_user(db, username).await?;

    // create login
    sqlx::query(
        r#"
        INSERT INTO logins (user_id, password_hash, salt)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(user.id)
    .bind(hashed)
    .bind(salt)
    .execute(db)
    .await?;

    Ok(())
}
