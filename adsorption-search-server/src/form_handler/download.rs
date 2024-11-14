use std::{fs, path::Path};

use axum::{body::Body, extract::Path as AxumPath, response::IntoResponse};

use hyper::header;

use super::error_handle::DownloadError;

/// Download result
pub async fn download_result(
    AxumPath(filename): AxumPath<String>,
) -> Result<impl IntoResponse, DownloadError> {
    let headers = [
        (header::CONTENT_TYPE, "application/zip"),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\".zip", &filename),
        ),
    ];
    let mut zip_path = Path::new("tempData").join(filename);
    zip_path.set_extension("zip");
    let body = Body::from(fs::read(&zip_path).unwrap());
    Ok((headers, body).into_response())
}
