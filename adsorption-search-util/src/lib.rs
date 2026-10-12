mod cell_model;
mod error;
mod run_params;

use castep_cell_fmt::parse;
use castep_cell_io::CellDocument;
use chemrust_nasl::SearchReports;
use rhino_lib::run;

use error::LoadError;
pub use run_params::{create_task_table, TaskTable};

#[derive(Debug, Clone, Copy)]
pub enum InputFormat {
    Msi,
    Cell,
    Cif,
}

/// Parses CASTEP `.cell` content into a 0.7.0 [`CellDocument`].
///
/// Only the `cell` extension is supported.
pub fn load_model(
    format_extension: &str,
    model_content: &str,
) -> Result<CellDocument, LoadError> {
    if let "cell" = format_extension {
        parse::<CellDocument>(model_content).map_err(|_| LoadError::ParseFailure)
    } else {
        Err(LoadError::UnsupportedFormat(format_extension.to_string()))
    }
}

/// Runs the adsorption site search described by `table` and returns a
/// short text report of the result.
///
/// The search runs through the `rhino` (chemrust-nasl) toolchain, which
/// supersedes the former `chemrust-scanner` `MountingChecker` API.
pub fn search_with_task_table(table: &TaskTable) -> String {
    let doc = match load_model("cell", table.model_content()) {
        Ok(doc) => doc,
        Err(err) => return err.to_string(),
    };
    let model = cell_model::CellModelAdapter::new(&doc);
    match run(&model, table) {
        Ok(report) => format_report(&report),
        Err(err) => err.to_string(),
    }
}

fn format_report(report: &SearchReports) -> String {
    let multi = report.points().map(|v| v.len()).unwrap_or(0);
    let single = report.viable_single_points().map(|v| v.len()).unwrap_or(0);
    let double = report.viable_double_points().map(|v| v.len()).unwrap_or(0);
    format!(
        "found {multi} multi-site, {single} single-site and {double} double-site candidates"
    )
}
