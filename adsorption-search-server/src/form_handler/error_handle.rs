use adsorption_search_ui::html_pages::upload_response_page::{
    build_error_format_page, build_error_symbol_page, build_form_error_general_page,
};
use axum::{
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};

#[derive(Debug)]
pub enum FormSubmitError {
    FormDataNotCollected,
    InvalidSymbol(String),
    NoFileName,
    InvalidFileFormat(FormatError),
    ParseFileError(FileError),
}

#[derive(Debug)]
pub enum FormatError {
    ExtensionNotFound,
    ExtensionNotAcceptable,
}

#[derive(Debug)]
pub enum FileError {
    NotStringContent,
    ParseFail,
}

pub const ACCEPT_EXTENSIONS: [&str; 3] = ["cif", "msi", "cell"];

impl IntoResponse for FormSubmitError {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidSymbol(symbol) => {
                Html(build_error_symbol_page(symbol).to_string()).into_response()
            }
            Self::FormDataNotCollected => Html("No form data collected").into_response(),
            Self::InvalidFileFormat(_) => {
                Redirect::to("/form_error/invalid_format").into_response()
            }
            Self::NoFileName => Redirect::to("/form_error/no_file_name").into_response(),
            Self::ParseFileError(_) => Redirect::to("/form_error/parse_error").into_response(),
        }
    }
}

pub fn error_router() -> Router {
    Router::new()
        .route("/invalid_format", get(format_error_page))
        .route("/no_file_name", get(no_file_name_error))
        .route("/parse_error", get(parse_error))
}

async fn format_error_page() -> impl IntoResponse {
    Html(build_error_format_page().to_string())
}

async fn no_file_name_error() -> impl IntoResponse {
    Html(
        build_form_error_general_page(
            "Error: filename is not valid!".into(),
            "The file does not have a filename in valid Unicode.".into(),
        )
        .to_string(),
    )
}

async fn parse_error() -> impl IntoResponse {
    Html(
        build_form_error_general_page(
            "Error: Failed to parse file".into(),
            "Please check if the file content matches the requirement of the file format.".into(),
        )
        .to_string(),
    )
}
