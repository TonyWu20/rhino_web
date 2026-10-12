use std::fmt::Display;

use adsorption_search_ui::html_pages::upload_response_page::{
    build_error_format_page, build_form_error_general_page,
};
use axum::{
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};

#[derive(Debug)]
pub enum FormSubmitError {
    NoFileName,
    InvalidFileFormat(FormatError),
    ParseFileError(FileError),
    ZipFileError,
    Execution,
}

impl Display for FormSubmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormSubmitError::NoFileName => f.write_str("No filename is given."),
            FormSubmitError::InvalidFileFormat(error) => write!(f, "{}", error),
            FormSubmitError::ParseFileError(p) => write!(f, "{:?}", p),
            FormSubmitError::ZipFileError => f.write_str("Error during preparing the zip file."),
            FormSubmitError::Execution => f.write_str("Error during Rhino execution."),
        }
    }
}
impl std::error::Error for FormSubmitError {}

#[derive(Debug)]
pub enum FormatError {
    ExtensionNotFound,
    ExtensionNotAcceptable,
}

impl Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::ExtensionNotFound => f.write_str("The extension is not found."),
            FormatError::ExtensionNotAcceptable => f.write_str("The extension is not acceptable."),
        }
    }
}

#[derive(Debug)]
pub enum FileError {
    NotStringContent,
    ParseFail,
}

impl IntoResponse for FormSubmitError {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidFileFormat(_) => {
                Redirect::to("/form_error/invalid_format").into_response()
            }
            Self::NoFileName => Redirect::to("/form_error/no_file_name").into_response(),
            Self::ParseFileError(_) => Redirect::to("/form_error/parse_error").into_response(),
            Self::Execution => Redirect::to("/form_error/execution_error").into_response(),
            Self::ZipFileError => Redirect::to("/form_error/execution_error").into_response(),
        }
    }
}

impl IntoResponse for FileError {
    fn into_response(self) -> Response {
        match self {
            Self::NotStringContent => {
                Redirect::to("/form_error/not_string_content").into_response()
            }
            Self::ParseFail => Redirect::to("/form_error/parse_error").into_response(),
        }
    }
}

/// Router for form error
pub fn form_error_router() -> Router {
    Router::new()
        .route("/invalid_format", get(format_error_page))
        .route("/no_file_name", get(no_file_name_error))
        .route("/parse_error", get(parse_error))
        .route("/not_string_content", get(not_string_content))
        .route("/execution_error", get(execution_error))
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

async fn not_string_content() -> impl IntoResponse {
    Html(
        build_form_error_general_page(
            "Error: uploaded content is not Unicode text".into(),
            "Please check the uploaded file.".into(),
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

async fn execution_error() -> impl IntoResponse {
    Html(
        build_form_error_general_page(
            "Error: Failed to complete the search".into(),
            "Rhino failed to complete the search or export results.".into(),
        )
        .to_string(),
    )
}
