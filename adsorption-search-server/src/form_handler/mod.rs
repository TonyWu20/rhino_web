use std::io::Read;

use adsorption_search_ui::html_pages::{build_error_symbol_page, build_upload_response_page};
use adsorption_search_util::{create_task_table, search_with_task_table};
use axum::{
    body::Body,
    extract::Request,
    http::header::CONTENT_TYPE,
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use axum_typed_multipart::TypedMultipart;
use http_body_util::BodyExt;

use castep_periodic_table::{data::ELEMENT_TABLE, element::LookupElement};
use form_data::{FormData, ParamForm};

mod form_data;

#[derive(Debug)]
pub enum ElementSymbolError {
    FormDataNotCollected,
    InvalidSymbol(String),
}

impl IntoResponse for ElementSymbolError {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidSymbol(symbol) => {
                Html(build_error_symbol_page(symbol).to_string()).into_response()
            }
            Self::FormDataNotCollected => Html("No form data collected").into_response(),
        }
    }
}

/// Layer to validate the param form input.
pub async fn validate_form(request: Request, next: Next) -> impl IntoResponse {
    let (parts, body) = request.into_parts();
    let content_type_header = parts.headers.get(CONTENT_TYPE);
    let content_type: Option<&str> = content_type_header.and_then(|value| value.to_str().ok());
    let boundary = content_type
        .unwrap()
        .strip_prefix("multipart/form-data; boundary=")
        .unwrap();
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => {
            return Err(ElementSymbolError::FormDataNotCollected);
        }
    };
    let uploaded_data = std::str::from_utf8(&bytes).unwrap();
    let element_symbol_line_prefix =
        "\r\nContent-Disposition: form-data; name=\"element_symbol\"\r\n";
    let element_symbol = uploaded_data
        .split(boundary)
        .into_iter()
        .filter(|s| s.starts_with(element_symbol_line_prefix))
        .next()
        .unwrap()
        .strip_prefix(element_symbol_line_prefix)
        .unwrap()
        .strip_prefix("\r\n")
        .unwrap()
        .strip_suffix("\r\n--")
        .unwrap();
    match ELEMENT_TABLE.get_by_symbol(element_symbol) {
        Some(_) => {
            let req = Request::from_parts(parts, Body::from(bytes));
            let response = next.run(req).await;
            Ok(response)
        }
        None => Err(ElementSymbolError::InvalidSymbol(element_symbol.into())),
    }
}

/// Handler to process form submission.
/// # Return
/// A page to show successful submission of the form.
pub async fn accept_form(
    TypedMultipart(FormData {
        file,
        element_symbol,
        target_bondlength,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    }): TypedMultipart<FormData>,
) -> impl IntoResponse {
    let mut model_file = file.contents.as_file();
    let mut model_content = String::new();
    model_file
        .read_to_string(&mut model_content)
        .expect("Unable to read file");
    let task_table = create_task_table(
        &model_content,
        element_symbol.clone(),
        target_bondlength,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    );
    let param_form = ParamForm {
        element_symbol,
        target_bondlength,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    };
    let _results = search_with_task_table(&task_table);
    let param_text = param_form.export_to_html();
    let page = build_upload_response_page(param_text).to_string();
    Html(page)
}
