//! HTML rendering.

use askama::Template;

use axum::response::{Html, IntoResponse, Response};

use http::StatusCode;

/// A HTML template to render as a response.
pub struct HtmlTemplate<T> {
    /// The template to render.
    pub template: T,
}

impl<T> HtmlTemplate<T> {
    pub fn new(template: T) -> HtmlTemplate<T> {
        HtmlTemplate { template }
    }
}

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.template.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
