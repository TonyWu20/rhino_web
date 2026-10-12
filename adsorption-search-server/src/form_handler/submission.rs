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
use castep_cell_fmt::{format::to_string_many_spaced, parse, ToCellFile};
use castep_cell_io::CellDocument;
use castep_cell_fmt::Error as CError;
use chemrust_core::data::lattice::CrystalModel;
use chemrust_nasl::{CoordSite, SearchReports, Visualize};
use crystal_cif_io::CifDocument;
use hyper::header;
use tempfile::NamedTempFile;
use zip::{
    write::{FileOptions, SimpleFileOptions},
    ZipWriter,
};

use super::{
    cell_model,
    error_handle::{FileError, FormSubmitError, FormatError},
    form_data::{FormData, ParamForm},
};
// use super::error_handle::ACCEPT_EXTENSIONS;
use castep_periodic_table::element::ElementSymbol;
use rhino_lib::{
    run,
    seeding::{ContentStorage, FileModel},
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
        AcceptExt::Cell => Ok(AcceptedFile::Cell(
            parse::<CellDocument>(&ret)
                .map_err(|_| FormSubmitError::ParseFileError(FileError::ParseFail))?,
        )),
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
    let results: Vec<(CellDocument, PathBuf)> = match file_content {
        AcceptedFile::Cif(cif_document) => {
            let datablock = cif_document.data_blocks().map(|v| v[0].clone()).unwrap();
            let search_report =
                run(&datablock, &param_form).map_err(|_| FormSubmitError::Execution)?;
            export_cif_sites(&datablock, &search_report, &param_form)
                .map_err(|_| FormSubmitError::Execution)?
        }

        AcceptedFile::Cell(doc) => {
            let model = cell_model::CellModelAdapter::new(&doc);
            let search_report =
                run(&model, &param_form).map_err(|_| FormSubmitError::Execution)?;
            export_cell_sites(&doc, &search_report, &param_form)
        }
    };
    let potentials = cell_model::potential_names(
        &results.iter().map(|(doc, _)| doc).collect::<Vec<&CellDocument>>(),
    );
    let all_potentials = cell_model::read_potentials(&potentials, param_form.potential_loc())
        .map_err(|_| FormSubmitError::Execution)?;
    let struct_files: Vec<FileModel> = results
        .into_iter()
        .map(|(doc, file_name)| {
            FileModel::new(
                file_name,
                to_string_many_spaced(&doc.to_cell_file()).into_bytes(),
            )
        })
        .collect();
    let potential_files: Vec<FileModel> = all_potentials
        .into_iter()
        .map(|(name, bytes)| {
            FileModel::new(Path::new(&name).to_path_buf(), bytes)
        })
        .collect();
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
            ContentStorage::new([struct_files, potential_files].concat()),
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

/// Check the file extension; if the file extension is not in the supported list `ACCEPT_EXTENSIONS`, return err;
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

/// Builds one export document per viable site for a parsed 0.7.0 cell
/// document. Optional blocks of the input document are preserved.
fn export_cell_sites(
    doc: &CellDocument,
    search_report: &SearchReports,
    param_form: &ParamForm,
) -> Vec<(CellDocument, PathBuf)> {
    let element = param_form.new_element();
    [
        build_cell_points(doc, search_report.viable_single_points(), element, param_form),
        build_cell_points(doc, search_report.viable_double_points(), element, param_form),
        build_cell_points(doc, search_report.points(), element, param_form),
    ]
    .concat()
}

fn build_cell_points<P: CoordSite + Visualize>(
    doc: &CellDocument,
    points: Option<&Vec<P>>,
    element: &ElementSymbol,
    param_form: &ParamForm,
) -> Vec<(CellDocument, PathBuf)> {
    let Some(v) = points else {
        return Vec::new();
    };
    v.iter()
        .map(|site| {
            let doc = cell_model::add_site_to_cell_document(doc, site, element);
            (doc, param_form.cell_export_filename(site))
        })
        .collect()
}

/// Builds one export document per viable site from a CIF data block.
fn export_cif_sites(
    model: &impl CrystalModel,
    search_report: &SearchReports,
    param_form: &ParamForm,
) -> Result<Vec<(CellDocument, PathBuf)>, CError> {
    let element = param_form.new_element();
    let parts = [
        build_cif_points(model, search_report.viable_single_points(), element, param_form)?,
        build_cif_points(model, search_report.viable_double_points(), element, param_form)?,
        build_cif_points(model, search_report.points(), element, param_form)?,
    ];
    Ok(parts.concat())
}

fn build_cif_points<P: CoordSite + Visualize>(
    model: &impl CrystalModel,
    points: Option<&Vec<P>>,
    element: &ElementSymbol,
    param_form: &ParamForm,
) -> Result<Vec<(CellDocument, PathBuf)>, CError> {
    let Some(v) = points else {
        return Ok(Vec::new());
    };
    v.iter()
        .map(|site| {
            let doc = cell_model::cell_document_from_model(model, element, site)?;
            Ok((doc, param_form.cell_export_filename(site)))
        })
        .collect()
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
    let out = zip_buf.into_inner();
    Ok(Bytes::from(out))
}
