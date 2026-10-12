use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use axum_typed_multipart::{FieldData, TryFromMultipart};
use castep_periodic_table::element::ElementSymbol;
use rhino_lib::{
    execution::{
        self,
        Axis::{X, Y, Z},
    },
    supportive_data::FractionalCoordRange,
    RhinoExport, SearchJob,
};
use serde::Deserialize;
use tempfile::NamedTempFile;

#[derive(Debug, TryFromMultipart)]
pub struct FormData {
    #[form_data(limit = "20MiB")]
    pub(crate) file: FieldData<NamedTempFile>,
    pub(crate) element_symbol: String,
    pub(crate) target_bondlength: f64,
    pub(crate) x_min: f64,
    pub(crate) x_max: f64,
    pub(crate) y_min: f64,
    pub(crate) y_max: f64,
    pub(crate) z_min: f64,
    pub(crate) z_max: f64,
}

#[derive(Debug, Deserialize)]
pub struct ParamForm {
    pub(crate) filename: String,
    pub(crate) element_symbol: ElementSymbol,
    pub(crate) target_bondlength: f64,
    pub(crate) x_min: f64,
    pub(crate) x_max: f64,
    pub(crate) y_min: f64,
    pub(crate) y_max: f64,
    pub(crate) z_min: f64,
    pub(crate) z_max: f64,
    pub(crate) potential_loc: PathBuf,
}

impl SearchJob for ParamForm {
    fn target_bondlength(&self) -> f64 {
        self.target_bondlength
    }

    fn search_range(
        &self,
        axis: execution::Axis,
    ) -> rhino_lib::supportive_data::FractionalCoordRange {
        match axis {
            X => FractionalCoordRange::new(self.x_min, self.x_max),
            Y => FractionalCoordRange::new(self.y_min, self.y_max),
            Z => FractionalCoordRange::new(self.z_min, self.z_max),
        }
    }
}

impl RhinoExport for ParamForm {
    fn new_element(&self) -> &castep_periodic_table::element::ElementSymbol {
        &self.element_symbol
    }

    fn base_model_name(&self) -> &str {
        &self.filename
    }

    fn export_filename<E: rhino_lib::ExportFormat>(
        &self,
        coord_site: &impl chemrust_nasl::CoordSite,
    ) -> PathBuf {
        let atom_ids_text = coord_site.connecting_atoms_msg();
        Path::new(&format!(
            "{}_{}.{}",
            self.base_model_name(),
            atom_ids_text,
            E::suffix()
        ))
        .to_path_buf()
    }

    fn export_dir(&self) -> &std::path::Path {
        todo!()
    }

    fn potential_loc(&self) -> &std::path::Path {
        &self.potential_loc
    }
}

impl ParamForm {
    pub fn target_bondlength(&self) -> f64 {
        self.target_bondlength
    }

    pub fn x_min(&self) -> f64 {
        self.x_min
    }

    pub fn x_max(&self) -> f64 {
        self.x_max
    }

    pub fn y_min(&self) -> f64 {
        self.y_min
    }

    pub fn y_max(&self) -> f64 {
        self.y_max
    }

    pub fn z_min(&self) -> f64 {
        self.z_min
    }

    pub fn z_max(&self) -> f64 {
        self.z_max
    }

    /// File name of a cell export for one coord site.
    pub fn cell_export_filename(&self, coord_site: &impl chemrust_nasl::CoordSite) -> PathBuf {
        let atom_ids_text = coord_site.connecting_atoms_msg();
        Path::new(&format!("{}_{}.cell", self.filename, atom_ids_text)).to_path_buf()
    }
    pub fn element_symbol(&self) -> ElementSymbol {
        self.element_symbol
    }
}

impl Display for ParamForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let element_line = format!("New Element Symbol: {}", self.element_symbol());
        let bondlength_line = format!("Target Bondlength (Å): {}", self.target_bondlength());
        let x_range_line = format!(
            "Search range along x-axis: {} - {}",
            self.x_min(),
            self.x_max()
        );
        let y_range_line = format!(
            "Search range along y-axis: {} - {}",
            self.y_min(),
            self.y_max()
        );
        let z_range_line = format!(
            "Search range along z-axis: {} - {}",
            self.z_min(),
            self.z_max()
        );
        let body = [
            element_line,
            bondlength_line,
            x_range_line,
            y_range_line,
            z_range_line,
        ];
        write!(f, "{}", body.join("\r\n"))
    }
}
