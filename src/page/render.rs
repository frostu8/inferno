//! Support for rendering pages.

use leptos::prelude::*;
use leptos::server_fn::codec::GetUrl;

use super::SendPage;

use crate::error::Error as ApiError;

/// The main page rendering endpoint.
#[server(endpoint = "/page", input = GetUrl, output = SendPage)]
pub async fn render_page(path: String) -> Result<Option<String>, ServerFnError<ApiError>> {
    use crate::ServerState;

    let state = expect_context::<ServerState>();

    #[derive(sqlx::FromRow)]
    struct Page {
        pub content: String,
    }

    // get page
    let page = sqlx::query_as::<_, Page>("SELECT content FROM pages WHERE path = $1")
        .bind(&path)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;

    if let Some(page) = page {
        // TODO complex page render logic
        Ok(Some(page.content))
    } else {
        Ok(None)
    }
}

/// Shows a page.
#[component]
pub fn Page(#[prop(into)] path: Signal<String>) -> impl IntoView {
    let content = Resource::new(move || path.get(), move |path| render_page(path));

    view! {
        <Suspense>
            {move || Suspend::new(async move {
                let content = content.await;

                view! {
                    <PageInner content=content />
                }
            })}
        </Suspense>
    }
}

#[component]
fn PageInner(content: Result<Option<String>, ServerFnError<ApiError>>) -> impl IntoView {
    match content {
        Ok(Some(content)) => view! { <RenderPage content/> }.into_any(),
        // TODO better error showing
        Ok(None) => view! { "not found" }.into_any(),
        Err(_) => view! { "error" }.into_any(),
    }
}

#[component]
fn RenderPage(content: String) -> impl IntoView {
    content.into_view()
}
