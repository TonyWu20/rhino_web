use std::fmt::Display;

use axum_typed_multipart::{FieldData, TryFromMultipart};
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
    pub(crate) element_symbol: String,
    pub(crate) target_bondlength: f64,
    pub(crate) x_min: f64,
    pub(crate) x_max: f64,
    pub(crate) y_min: f64,
    pub(crate) y_max: f64,
    pub(crate) z_min: f64,
    pub(crate) z_max: f64,
}

impl ParamForm {
    pub fn element_symbol(&self) -> &str {
        self.element_symbol.as_ref()
    }

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
    pub fn export_to_html(&self) -> String {
        let element_line = format!("New Element Symbol: {}", self.element_symbol());
        let bondlength_line = format!("Target Bondlength (Å): {}", self.target_bondlength());
        let x_range_line = format!(
            "Search range along x-axis: {:.3} - {:.3}",
            self.x_min(),
            self.x_max()
        );
        let y_range_line = format!(
            "Search range along y-axis: {:.3} - {:.3}",
            self.y_min(),
            self.y_max()
        );
        let z_range_line = format!(
            "Search range along z-axis: {:.3} - {:.3}",
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
        body.join("<br>")
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
