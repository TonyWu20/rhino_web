#![recursion_limit = "512"]
use adsorption_search_ui::{css_styles::build_css_sheet, html_pages::root_page::build_root_page};
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use form_handler::accept_form;
use std::{fs, net::SocketAddr, path::Path};
use tokio::signal;
use tower_http::{
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    services::ServeDir,
    trace::{self, TraceLayer},
};
use tracing::{error, info, Level};

use crate::form_handler::{download_result, error_handle::form_error_router};

mod form_handler;
mod model_search_run;

#[tokio::main]
async fn main() {
    // Route all requests on "/" endpoint to anonymous handler.
    // A handler is an async function which returns something that implements
    // `axum::response::IntoResponse`.
    // A closure or a function can be used as handler.

    tracing_subscriber::fmt().with_target(true).compact().init();
    let temp_data_path = Path::new("tempData");
    if !temp_data_path.exists() {
        fs::create_dir(temp_data_path).unwrap();
    }
    let assets_path = Path::new("assets");
    if !assets_path.exists() {
        error!("Run this program in the directory with the provided `assets` folder included.");
        panic!("Run this program in the directory with the provided `assets` folder included.")
    }
    let app: Router<()> = Router::new()
        .route("/", get(root_page))
        .nest("/form_error", form_error_router())
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/tempData", ServeDir::new("tempData"))
        .route("/submit_form", post(accept_form))
        .route("/download_file/:filename", get(download_result))
        .route("/my_wasm.js", get(load_js_wasm))
        .route("/new_style.css", get(css_sheet))
        .layer(RequestBodyLimitLayer::new(
            20 * 1024 * 1024, // 20mb
        ))
        .layer(CompressionLayer::new())
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

    // let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let addr = SocketAddr::from(([0, 0, 0, 0], 10000));
    // let addr = SocketAddr::from(([192, 168, 1, 106], 3000));
    // let addr = SocketAddr::from(([10, 147, 17, 25], 3000));
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("listening on {}", addr);
    info!(
        "Please enter the address in your browser to access: {}",
        addr
    );

    // Use `hyper::server::Server` which is re-exported through `axum::Server` to serve the app.
    axum::serve(listener, app.into_make_service())
        // .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn root_page() -> impl IntoResponse {
    Html(build_root_page().to_string())
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
        .body(include_str!("../../assets/my_wasm.js").to_string())
        .unwrap()
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
