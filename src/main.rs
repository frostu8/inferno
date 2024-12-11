#[cfg(feature = "ssr")]
use anyhow::Error as AnyhowError;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), AnyhowError> {
    use axum::Router;
    use inferno::{app::*, cli::Cli, ServerState};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::env;

    #[cfg(feature = "ssr")]
    dotenv::dotenv().ok();

    // Create shared server state
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            log!("failed to get `DATABASE_URL`");
            std::process::exit(1);
        }
    };
    let state = ServerState::new(&database_url).await?;

    // try expose cli
    Cli::parse().run(&state).await;

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
