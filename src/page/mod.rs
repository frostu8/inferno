//! Pages, the top-level construct of inferno.
//!
//! This module contains some shared page codecs and components. All the render
//! logic has been sectioned off in `render` since it is quite complicated.

pub mod render;

use leptos::prelude::*;
use leptos::server_fn::codec::{Encoding, FromRes, IntoRes};
use leptos::server_fn::response::ClientRes;
use leptos::server_fn::ServerFnError;

#[cfg(not(feature = "ssr"))]
use leptos::server_fn::response::BrowserMockRes;

#[cfg(feature = "ssr")]
use axum::body::Body;
#[cfg(feature = "ssr")]
use axum::response::Response;

use http::Method;

/// Codec for page endpoints.
pub struct SendPage;

impl Encoding for SendPage {
    const CONTENT_TYPE: &'static str = "text/html; charset=utf-8";

    const METHOD: Method = Method::GET;
}

#[cfg(feature = "ssr")]
impl<CustErr> IntoRes<SendPage, Response<Body>, CustErr> for Option<String> {
    async fn into_res(self) -> Result<Response<Body>, ServerFnError<CustErr>> {
        use axum::http::{header, StatusCode};

        if let Some(content) = self {
            Ok(Response::builder()
                .header(header::CONTENT_TYPE, SendPage::CONTENT_TYPE)
                .body(Body::from(content))
                .unwrap())
        } else {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::default())
                .unwrap())
        }
    }
}

#[cfg(not(feature = "ssr"))]
impl<CustErr> IntoRes<SendPage, BrowserMockRes, CustErr> for Option<String> {
    async fn into_res(self) -> Result<BrowserMockRes, ServerFnError<CustErr>> {
        unreachable!();
    }
}

impl<CustErr, Response> FromRes<SendPage, Response, CustErr> for Option<String>
where
    Response: ClientRes<CustErr> + Send,
{
    async fn from_res(res: Response) -> Result<Self, ServerFnError<CustErr>> {
        if res.status() == 404 {
            Ok(None)
        } else {
            Ok(Some(res.try_into_string().await?))
        }
    }
}

#[component]
pub fn Page(#[prop(into)] path: Signal<String>) -> impl IntoView {
    let content = Resource::new(move || path.get(), move |path| render::render_page(path));

    view! {
        <Suspense>
            {move || Suspend::new(async move {
                let content = content.await;

                match content {
                    Ok(Some(content)) => view! { <RenderPage content/> }.into_any(),
                    // TODO better error showing
                    Ok(None) => view! { "not found" }.into_any(),
                    Err(_) => view! { "error" }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
fn RenderPage(content: String) -> impl IntoView {
    view! { <div class="page-content" inner_html=content></div> }
}
