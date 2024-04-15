#![recursion_limit = "512"]
use std::net::SocketAddr;

use adsorption_search_ui::{css_styles::build_css_sheet, html_pages::build_root_page};
use axum::{
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use form_handler::{accept_form, validate_form};
use tower_http::{
    limit::RequestBodyLimitLayer,
    services::ServeDir,
    trace::{self, TraceLayer},
};
use tracing::Level;

mod form_handler;
mod model_search_run;

#[tokio::main]
async fn main() {
    // Route all requests on "/" endpoint to anonymous handler.
    // A handler is an async function which returns something that implements
    // `axum::response::IntoResponse`.
    // A closure or a function can be used as handler.

    tracing_subscriber::fmt().with_target(false).pretty().init();

    let app: Router<()> = Router::new()
        .route("/", get(root_page))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/tempData", ServeDir::new("tempData"))
        .route(
            "/submit_form",
            post(accept_form).route_layer(middleware::from_fn(validate_form)),
        )
        .route("/js-wasm.js", get(load_js_wasm))
        .route("/new_style.css", get(css_sheet))
        .layer(RequestBodyLimitLayer::new(
            20 * 1024 * 1024, // 20mb
        ))
        // Applies a tower::Layer to all routes in the router
        // Adds high level tracing to service
        .layer(
            TraceLayer::new_for_http() // HTTP responses
                // Customizes how to make `Span`s that all request handling will be wrapped in
                // Default level is `Level::DEBUG`
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                // Customizes what to do when a response has been produced
                // Default level is `Level::DEBUG`
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", addr);

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

async fn load_js_wasm() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/javascript")
        .body(include_str!("../../js-wasm.js").to_string())
        .unwrap()
}
