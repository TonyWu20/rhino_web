use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use adsorption_search_ui::html_pages::upload_response_page::build_upload_response_page;
use adsorption_search_util::{
    create_task_table, filtered_atoms, init_checker, load_model, search_with_task_table,
};
use axum::response::{Html, IntoResponse};
use axum_typed_multipart::{FieldMetadata, TypedMultipart};

use super::{
    error_handle::{FileError, FormSubmitError, FormatError, ACCEPT_EXTENSIONS},
    form_data::{FormData, ParamForm},
};
use castep_periodic_table::{data::ELEMENT_TABLE, element::LookupElement};
use zip::{write::SimpleFileOptions, ZipWriter};

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
    let file_name: String = validate_filename(file.metadata)?;
    let file_path = Path::new(&file_name);
    // Check extension
    let extension = validate_extension(file_path)?;
    // Check if the symbol input is valid.
    let symbol = ELEMENT_TABLE
        .get_by_symbol(&element_symbol)
        .ok_or(FormSubmitError::InvalidSymbol(element_symbol.clone()))?;
    // Open the file, try to read to string and parse.
    let mut model_file = file.contents.as_file();
    let mut model_content = String::new();
    model_file
        .read_to_string(&mut model_content)
        .map_err(|_| FormSubmitError::ParseFileError(FileError::NotStringContent))?;
    let checker = init_checker(symbol, target_bondlength);
    let lattice_model = load_model(extension, &model_content)
        .map_err(|_| FormSubmitError::InvalidFileFormat(FormatError::ExtensionNotAcceptable))?;
    let filtered_atoms = filtered_atoms(&lattice_model, x_min, x_max, y_min, y_max, z_min, z_max);
    let results = checker
        .mount_search(lattice_model.atoms(), &filtered_atoms)
        .report_spheres();
    let param_form = ParamForm {
        element_symbol: element_symbol.clone(),
        target_bondlength,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    };

    let result_filename = format!(
        "{}_{}_{}",
        file_path.file_stem().unwrap().to_str().unwrap(),
        element_symbol,
        chrono::offset::Utc::now().timestamp()
    );
    // handle result output
    let tmp_path = Path::new("tempData");
    let mut txt_path = tmp_path.join(&result_filename);
    txt_path.set_extension("txt");
    fs::write(txt_path.clone(), results).expect("Write to tmp failed!");
    // Prepare zip to be downnload
    let mut zip_path = tmp_path.join(&result_filename);
    zip_path.set_extension("zip");
    let file = File::create(zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    zip.start_file(
        txt_path.to_str().unwrap(),
        SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored),
    )
    .unwrap();
    zip.finish().unwrap();
    // Remove temp files
    fs::remove_file(txt_path).unwrap();
    // Build response page
    let param_text = param_form.export_to_html();
    let page = build_upload_response_page(param_text, result_filename).to_string();
    Ok(Html(page))
}

fn validate_filename(field: FieldMetadata) -> Result<String, FormSubmitError> {
    field.file_name.ok_or(FormSubmitError::NoFileName)
}
/// Check the file extension; if the extension is not in the supported list `ACCEPT_EXTENSIONS`, return err;
fn validate_extension(file_path: &Path) -> Result<&str, FormSubmitError> {
    let file_extension = file_path
        .extension()
        .ok_or(FormSubmitError::InvalidFileFormat(
            FormatError::ExtensionNotFound,
        ))?
        .to_str()
        .unwrap_or("n/a"); // Fallback when the `OsStr` is not valid Unicode.
    if !ACCEPT_EXTENSIONS.contains(&file_extension) {
        Err(FormSubmitError::InvalidFileFormat(
            FormatError::ExtensionNotAcceptable,
        ))
    } else {
        Ok(file_extension)
    }
}
