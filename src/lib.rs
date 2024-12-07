pub mod account;
pub mod app;

/// Shared server state.
///
/// Cheaply cloneable.
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct ServerState {
    /// The secret signing key for tokens.
    ///
    /// This is randomly generated on app startup. This means that when the
    /// daemon restarts, old JWTs will be rejected.
    pub jwt_secret: std::sync::Arc<str>,
}

#[cfg(feature = "ssr")]
impl ServerState {
    /// Creates a new `ServerState`.
    pub fn new() -> ServerState {
        use base16::encode_lower;
        use rand::{rngs::StdRng, Rng, SeedableRng};
        use std::sync::Arc;

        // randomly generate JWT secret
        let mut rng = StdRng::from_entropy();

        let mut bytes = [0u8; 256];
        rng.fill(&mut bytes);

        let jwt_secret = Arc::from(encode_lower(&bytes));

        ServerState { jwt_secret }
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
