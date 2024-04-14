use axum_typed_multipart::{FieldData, TryFromMultipart, TypedMultipart};
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
}
