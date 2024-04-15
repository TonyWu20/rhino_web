#![recursion_limit = "512"]
use adsorption_search_ui::{css_styles::build_css_sheet, html_pages::build_root_page};
use axum::{
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use form_handler::{accept_form, validate_form};
use tower_http::{limit::RequestBodyLimitLayer, services::ServeDir};

mod form_handler;
mod model_search_run;

#[tokio::main]
async fn main() {
    // Route all requests on "/" endpoint to anonymous handler.
    // A handler is an async function which returns something that implements
    // `axum::response::IntoResponse`.
    // A closure or a function can be used as handler.

    let app: Router<()> = Router::new()
        .route("/submit_form", post(accept_form))
        .route_layer(middleware::from_fn(validate_form))
        .route("/", get(root_page))
        .route("/new_style.css", get(css_sheet))
        .nest_service("/tempData", ServeDir::new("tempData"))
        .layer(RequestBodyLimitLayer::new(
            20 * 1024 * 1024, // 20mb
        ));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    // Use `hyper::server::Server` which is re-exported through `axum::Server` to serve the app.
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn root_page() -> impl IntoResponse {
    let html = build_root_page().to_string();
    Html(html)
}

async fn css_sheet() -> impl IntoResponse {
    let css_sheet = build_css_sheet();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/css")
        .body(format!("{}", css_sheet))
        .unwrap()
}
