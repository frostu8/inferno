#[cfg(feature = "ssr")]
use anyhow::Error as AnyhowError;

#[cfg(feature = "ssr")]
// HACK something in tokio::main is causing this lint to raise
#[allow(clippy::needless_return)]
#[tokio::main]
async fn main() -> Result<(), AnyhowError> {
    use axum::Router;
    use inferno::{app::*, cli::Cli, ServerStateConfig};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::env;

    dotenv::dotenv().ok();

    // Create shared server state
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            log!("failed to get `DATABASE_URL`");
            std::process::exit(1);
        }
    };

    let mut config = ServerStateConfig::new().database_url(database_url);

    if let Ok(key) = env::var("TOKEN_SIGNING_KEY") {
        config = config.signing_key(key);
    }

    let state = config.build().await?;

    // try expose cli
    Cli::parse().run(&state).await;

    // embed migrations
    if let Err(err) = sqlx::migrate!("./migrations").run(&state.pool).await {
        log!("failed to run migrations: {}", err);
        log!("a database recreation may be necessary");
        std::process::exit(1);
    }

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(state.clone()),
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(AnyhowError::from)
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
