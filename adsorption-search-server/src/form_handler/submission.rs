use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

// use adsorption_search_ui::html_pages::upload_response_page::build_upload_response_page;
use axum::{
    body::{Body, Bytes},
    response::IntoResponse,
};
use axum_typed_multipart::{FieldData, FieldMetadata, TypedMultipart};
use castep_cell_io::{CellDocument, CellParser};
use chemrust_core::data::lattice::CrystalModel;
use chemrust_nasl::SearchReports;
use crystal_cif_io::CifDocument;
use hyper::header;
use tempfile::NamedTempFile;
use zip::{
    write::{FileOptions, SimpleFileOptions},
    ZipWriter,
};

use super::{
    error_handle::{FileError, FormSubmitError, FormatError},
    form_data::{FormData, ParamForm},
};
// use super::error_handle::ACCEPT_EXTENSIONS;
use castep_periodic_table::element::ElementSymbol;
use rhino_lib::{
    execution::ExportFile,
    run,
    seeding::{get_all_potentials, ContentStorage, FileModel},
    RhinoExport,
};

pub enum AcceptedFile {
    Cif(CifDocument),
    Cell(CellDocument),
}

/// # TODO
/// 1. A function to retrive parameter and into a config
/// 2. A function to call the `chemrust-nasl` by passing in the config
pub fn check_file_and_read_to_string(
    mut file: FieldData<NamedTempFile>,
) -> Result<AcceptedFile, FormSubmitError> {
    // Check the filename; if the file metadata do not have a filename, return err;
    let file_name: String = validate_filename(&file.metadata)?;
    let file_path = Path::new(&file_name);
    // Check extension
    let ext = validate_extension(file_path)?;
    let mut ret = String::new();
    file.contents
        .read_to_string(&mut ret)
        .map_err(|_| FormSubmitError::ParseFileError(FileError::NotStringContent))?;
    match ext {
        AcceptExt::Cell => {
            let mut parser = CellParser::from(&ret);
            Ok(AcceptedFile::Cell(parser.parse().map_err(|_| {
                FormSubmitError::ParseFileError(FileError::ParseFail)
            })?))
        }
        AcceptExt::Cif => Ok(AcceptedFile::Cif(
            CifDocument::parse_from_str(&mut ret.as_str())
                .map_err(|_| FormSubmitError::ParseFileError(FileError::ParseFail))?,
        )),
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
) -> Result<impl IntoResponse, FormSubmitError> {
    let file_name = Path::new(&validate_filename(&file.metadata)?)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let file_content = check_file_and_read_to_string(file)?;
    let param_form = ParamForm {
        filename: file_name.clone(),
        element_symbol: ElementSymbol::from_str(&element_symbol).unwrap(),
        target_bondlength,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
        potential_loc: Path::new("Potentials").to_path_buf(),
    };
    // Execute
    let results = match file_content {
        AcceptedFile::Cif(cif_document) => {
            let datablock = cif_document.data_blocks().map(|v| v[0].clone()).unwrap();
            let search_report =
                run(&datablock, &param_form).map_err(|_| FormSubmitError::Execution)?;
            export_to_mem(datablock, &search_report, &param_form).await
        }

        AcceptedFile::Cell(cell_document) => {
            let search_report = run(&cell_document, &param_form);
            let search_report = search_report.map_err(|_| FormSubmitError::Execution)?;
            export_to_mem(cell_document, &search_report, &param_form).await
        }
    };
    let all_potentials = get_all_potentials(&results, param_form.potential_loc())
        .map_err(|_| FormSubmitError::Execution)?
        .into_iter()
        .map(|(name, potential_file)| {
            FileModel::new(Path::new(&name).to_path_buf(), potential_file.0)
        })
        .collect::<Vec<FileModel>>();
    let struct_files = results
        .into_iter()
        .map(|c| {
            FileModel::new(
                c.file_name().to_path_buf(),
                c.file().to_string().into_bytes(),
            )
        })
        .collect::<Vec<FileModel>>();
    let seed_apps = [
        (
            "seeding-apps/arm64-macOS/castep-seeding-app",
            "castep-seeding-app-macOS-arm64",
        ),
        (
            "seeding-apps/x86_64-unknown-linux-gnu/release/castep-seeding-app",
            "castep-seeding-app-linux-gnu-gcc",
        ),
        (
            "seeding-apps/x86_64-pc-windows-gnu/release/castep-seeding-app.exe",
            "castep-seeding-app-win64-gnu.exe",
        ),
    ]
    .into_iter()
    .map(|(src, dst)| {
        Ok(FileModel::new(
            dst.into(),
            fs::read(src).map_err(|_| FormSubmitError::NoFileName)?,
        ))
    })
    .collect::<Result<Vec<FileModel>, FormSubmitError>>()?;

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644)
        .large_file(true);
    let app_options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755)
        .large_file(true);
    let seed_files = [
        (ContentStorage::new(seed_apps), app_options),
        (
            ContentStorage::new([struct_files, all_potentials].concat()),
            options,
        ),
    ]
    .to_vec();
    // handle result output
    // Prepare zip to be downnload
    let zip_file = [create_zip(seed_files)
        .await
        .map_err(|_| FormSubmitError::ZipFileError)?]
    .concat();
    // let it = WalkDir::new(task_table.export_dir()).into_iter();
    // zip_dir(
    //     &mut it.filter_map(|e| e.ok()),
    //     task_table.export_dir(),
    //     zip_file,
    //     zip::CompressionMethod::DEFLATE,
    // )
    // .map_err(|_| FormSubmitError::ZipFileError)?;
    // Build response page
    let headers = [
        (header::CONTENT_TYPE, "application/zip"),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}_results\".zip", &file_name),
        ),
    ];
    let body = Body::from(zip_file);
    let download_response = (headers, body).into_response();
    // let param_text = param_form.export_to_html();
    // let page = build_upload_response_page(
    //     param_text,
    //     zip_path.file_stem().unwrap().to_string_lossy().to_string(),
    // )
    // .to_string();
    Ok(download_response)
}

fn validate_filename(field: &FieldMetadata) -> Result<String, FormSubmitError> {
    field.file_name.clone().ok_or(FormSubmitError::NoFileName)
}

enum AcceptExt {
    Cif,
    Cell,
}

/// Check the file extension; if the extension is not in the supported list `ACCEPT_EXTENSIONS`, return err;
fn validate_extension(file_path: &Path) -> Result<AcceptExt, FormSubmitError> {
    let file_extension = file_path
        .extension()
        .ok_or(FormSubmitError::InvalidFileFormat(
            FormatError::ExtensionNotFound,
        ))?
        .to_str()
        .unwrap_or("n/a"); // Fallback when the `OsStr` is not valid Unicode.
    match file_extension {
        "cif" => Ok(AcceptExt::Cif),
        "cell" => Ok(AcceptExt::Cell),
        _ => Err(FormSubmitError::InvalidFileFormat(
            FormatError::ExtensionNotAcceptable,
        )),
    }
}

async fn export_to_mem(
    base_model: impl CrystalModel,
    search_report: &SearchReports,
    param_form: &ParamForm,
) -> Vec<ExportFile<CellDocument, PathBuf>> {
    let part_single_points = search_report
        .viable_single_points()
        .map(|points| param_form.create_all_sites::<CellDocument>(&base_model, points));
    let part_double = search_report
        .viable_double_points()
        .map(|points| param_form.create_all_sites::<CellDocument>(&base_model, points));
    let part_multi = search_report
        .points()
        .map(|points| param_form.create_all_sites::<CellDocument>(&base_model, points));
    [
        // If it is `None`, `unwrap_or_default` returns an empty Vec
        part_single_points,
        part_double,
        part_multi,
    ]
    .into_iter()
    .flat_map(|part| part.unwrap_or_default())
    .collect::<Vec<ExportFile<CellDocument, PathBuf>>>()
}

async fn create_zip(
    files: Vec<(ContentStorage, FileOptions<'_, ()>)>,
) -> Result<Bytes, anyhow::Error> {
    let mut zip_buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(&mut zip_buf);
    files
        .into_iter()
        .try_for_each(|(contents, option)| -> Result<(), anyhow::Error> {
            contents
                .files()
                .iter()
                .try_for_each(|file| -> Result<(), anyhow::Error> {
                    let name = file.path();
                    let data = file.data();
                    zip.start_file_from_path(name, option)?;
                    zip.write_all(data)?;
                    Ok(())
                })?;
            Ok(())
        })?;
    zip.finish()?;
    Ok(Bytes::from(zip_buf.into_inner()))
}
