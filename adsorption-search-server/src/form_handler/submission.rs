use std::{
    fs::{self, File},
    io::{Read, Seek, Write},
    path::Path,
    str::FromStr,
};

use adsorption_search_ui::html_pages::upload_response_page::build_upload_response_page;
use axum::response::{Html, IntoResponse};
use axum_typed_multipart::{FieldMetadata, TypedMultipart};
use std::error::Error;
use zip::write::SimpleFileOptions;

use super::{
    error_handle::{FormSubmitError, FormatError, ACCEPT_EXTENSIONS},
    form_data::{FormData, ParamForm},
};
use castep_periodic_table::element::ElementSymbol;
use rhino_lib::{run_by_table, TaskTable};
use walkdir::{DirEntry, WalkDir};

/// # TODO
/// 1. A function to retrive parameter and into a config
/// 2. A function to call the `chemrust-nasl` by passing in the config
pub fn create_task_table(
    TypedMultipart(FormData {
        mut file,
        element_symbol,
        target_bondlength,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    }): TypedMultipart<FormData>,
) -> Result<TaskTable, FormSubmitError> {
    // Check the filename; if the file metadata do not have a filename, return err;
    let file_name: String = validate_filename(file.metadata)?;
    let file_path = Path::new(&file_name);
    // Check extension
    validate_extension(file_path)?;
    // Check if the symbol input is valid.
    let symbol = ElementSymbol::from_str(&element_symbol)
        .map_err(|_| FormSubmitError::InvalidSymbol(element_symbol.clone()))?;
    let result_id = format!(
        "{}_{}_{}",
        file_path.file_stem().unwrap().to_str().unwrap(),
        element_symbol,
        chrono::offset::Utc::now().timestamp()
    );
    let temp_dir = Path::new("tempData");
    let export_dir = temp_dir.join(result_id);
    let temp_model_path = temp_dir.join(&file_name);
    // file.contents
    //     .persist(&temp_model_path)
    //     .map_err(|_| FormSubmitError::FormDataNotCollected)?;
    let mut temp_file = File::create_new(temp_model_path.clone()).unwrap();
    let mut buf = vec![];
    file.contents.read_to_end(&mut buf).unwrap();
    temp_file.write_all(&buf).unwrap();

    let task_table = rhino_lib::TaskTable::new(
        temp_model_path.to_string_lossy().to_string(),
        symbol,
        target_bondlength,
        (x_min, x_max),
        (y_min, y_max),
        (z_min, z_max),
        export_dir,
        None,
        rhino_lib::KPointQuality::Coarse,
        false,
    );
    Ok(task_table)
}

/// Handler to process form submission.
/// # Return
/// A page to show successful submission of the form.
pub async fn accept_form(
    TypedMultipart(FormData {
        mut file,
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
    let task_table = create_task_table(axum_typed_multipart::TypedMultipart(FormData {
        file,
        element_symbol: element_symbol.clone(),
        target_bondlength,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    }))?;
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
    // Execute
    run_by_table(&task_table).map_err(|e| {
        dbg!(e);
        FormSubmitError::Execution
    })?;
    fs::remove_file(task_table.model_path()).map_err(|_| FormSubmitError::NoFileName)?;
    // handle result output
    // Prepare zip to be downnload
    let mut zip_path = task_table.export_dir().clone();
    zip_path.set_extension("zip");
    let zip_file = File::create(&zip_path).unwrap();
    let it = WalkDir::new(task_table.export_dir()).into_iter();
    zip_dir(
        &mut it.filter_map(|e| e.ok()),
        task_table.export_dir(),
        zip_file,
        zip::CompressionMethod::DEFLATE,
    )
    .map_err(|_| FormSubmitError::ZipFileError)?;
    // Build response page
    let param_text = param_form.export_to_html();
    let page = build_upload_response_page(
        param_text,
        zip_path.file_stem().unwrap().to_string_lossy().to_string(),
    )
    .to_string();
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

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) -> Result<(), Box<dyn Error>>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .ok_or(FormSubmitError::ZipFileError)?;
        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path_as_string:?} as {name:?} ...");
            zip.add_directory(path_as_string, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}
