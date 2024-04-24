use adsorption_search_ui::html_pages::upload_response_page::build_form_error_general_page;
use axum::response::{Html, IntoResponse, Response};

#[derive(Debug)]
pub enum DownloadError {
    ZipCreateError(String),
}

impl IntoResponse for DownloadError {
    fn into_response(self) -> Response {
        match self {
            Self::ZipCreateError(message) => zip_create_error(&message).into_response(),
        }
    }
}

fn zip_create_error(message: &str) -> impl IntoResponse {
    Html(
        build_form_error_general_page(
            "Error: failed to create a zip for the generated file".into(),
            message.to_string(),
        )
        .to_string(),
    )
}
