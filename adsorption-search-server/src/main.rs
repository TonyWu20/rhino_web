use axum::{response::Html, routing::get, serve::Serve, Router};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    // Route all requests on "/" endpoint to anonymous handler.
    // A handler is an async function which returns something that implements
    // `axum::response::IntoResponse`.
    // A closure or a function can be used as handler.

    let app: Router<()> = Router::new()
        .route("/", get(handler))
        .nest_service("/assets", ServeDir::new("assets"));
    //        Router::new().route("/", get(|| async { "Hello, world!"}));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    // Use `hyper::server::Server` which is re-exported through `axum::Server` to serve the app.
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html(include_str!("../index.html"))
}

fn using_serve_dir() -> Router {
    Router::new().nest_service("/assets", ServeDir::new("assets"))
}
