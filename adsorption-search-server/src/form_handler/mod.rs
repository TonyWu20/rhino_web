use std::{io::Read, path::Path};

use adsorption_search_ui::html_pages::upload_response_page::build_upload_response_page;
use adsorption_search_util::{create_task_table, search_with_task_table};
use axum::response::{Html, IntoResponse};
use axum_typed_multipart::TypedMultipart;

use castep_periodic_table::{data::ELEMENT_TABLE, element::LookupElement};
use error_handle::FormSubmitError;
use form_data::{FormData, ParamForm};

use self::error_handle::{FileError, FormatError, ACCEPT_EXTENSIONS};

pub mod error_handle;
mod form_data;

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
) -> Result<impl IntoResponse, FormSubmitError> {
    // Check the filename; if the file metadata do not have a filename, return err;
    let file_name: String = file.metadata.file_name.ok_or(FormSubmitError::NoFileName)?;
    // Check the file extension; if the extension is not in the supported list `ACCEPT_EXTENSIONS`, return err;
    let file_extension = Path::new(&file_name)
        .extension()
        .ok_or(FormSubmitError::InvalidFileFormat(
            FormatError::ExtensionNotFound,
        ))?
        .to_str()
        .unwrap_or("n/a"); // Fallback when the `OsStr` is not valid Unicode.
    if !ACCEPT_EXTENSIONS.contains(&file_extension) {
        return Err(FormSubmitError::InvalidFileFormat(
            FormatError::ExtensionNotAcceptable,
        ));
    }
    // Open the file, try to read to string and parse.
    let mut model_file = file.contents.as_file();
    let mut model_content = String::new();
    model_file
        .read_to_string(&mut model_content)
        .map_err(|_| FormSubmitError::ParseFileError(FileError::NotStringContent))?;
    // Check if the symbol input is valid.
    ELEMENT_TABLE
        .get_by_symbol(&element_symbol)
        .ok_or(FormSubmitError::InvalidSymbol(element_symbol.clone()))?;
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
    Ok(Html(page))
}
