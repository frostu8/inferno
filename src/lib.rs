pub mod account;
pub mod app;
pub mod error;
pub mod page;
pub mod user;

#[cfg(feature = "ssr")]
pub mod cli;
#[cfg(feature = "ssr")]
pub mod passwords;
#[cfg(feature = "ssr")]
pub mod schema;
#[cfg(feature = "ssr")]
pub mod server;

#[cfg(feature = "ssr")]
pub use server::*;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
