use std::io::Read;

use adsorption_search_ui::build_upload_response_page;
use adsorption_search_util::{create_task_table, search_with_task_table};
use axum::{
    body::{Body, Bytes},
    extract::{Multipart, Request},
    http::StatusCode,
    middleware::Next,
    response::{self, Html, IntoResponse, Response},
    RequestExt,
};
use axum_typed_multipart::TypedMultipart;
use http_body_util::BodyExt;

use castep_periodic_table::{data::ELEMENT_TABLE, element::LookupElement};
use form_data::{FormData, ParamForm};

mod form_data;

pub async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    println!("{:#?}", bytes);
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}

/// Layer to validate the param form input.
pub async fn validate_form(
    data: TypedMultipart<FormData>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // request.body();
    // if let None = ELEMENT_TABLE.get_by_symbol(&element_symbol) {
    //     return Err(StatusCode::UNPROCESSABLE_ENTITY);
    // }
    // let param_form = ParamForm {
    //     element_symbol,
    //     target_bondlength,
    //     x_min,
    //     x_max,
    //     y_min,
    //     y_max,
    //     z_min,
    //     z_max,
    // };
    // let param_text = format!("{:#?}", param_form);
    // let page = build_upload_response_page(param_text).to_string();
    let response = next.run(request).await;
    Ok(response)
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
        element_symbol,
        target_bondlength,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    );
    let results = search_with_task_table(&task_table);
    let text = format!("{:#?}", results);
    Html(text)
}
