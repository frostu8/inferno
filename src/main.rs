#![forbid(unsafe_code)]

use color_eyre::Section;
use eyre::Report;

use inferno::{
    cli::{Cli, ShouldContinue},
    read_config, routes,
};

use tracing::info;

use tower_http::{services::ServeDir, trace::TraceLayer};

use std::net::SocketAddr;

// HACK something in tokio::main is causing this lint to raise
#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() -> Result<(), Report> {
    install_tracing();
    color_eyre::install()?;
    dotenv::dotenv().ok();

    // Create shared server state
    let config = read_config()?;
    let state = config.build().await?;

    // try expose cli
    match Cli::parse().run(&state).await? {
        ShouldContinue::Daemon => (),
        ShouldContinue::Exit => std::process::exit(0),
    }

    // embed migrations
    if let Err(err) = sqlx::migrate!("./migrations").run(&state.pool).await {
        return Err(Report::msg(format!("failed to run migrations: {}", err))
            .note("a database recreation may be necessary"));
    }

    // TODO proper port/addr stuff
    let addr: SocketAddr = ([0, 0, 0, 0], state.port).into();

    let static_dir = ServeDir::new(&state.static_files_dir);
    let routes = routes::all()
        .with_state(state)
        .fallback_service(static_dir)
        .layer(TraceLayer::new_for_http());

    info!("listening on {}...", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, routes).await.map_err(Report::from)
}

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
