use castep_periodic_table::{
    data::ELEMENT_TABLE,
    element::{Element, ElementSymbol, LookupElement},
};
use rhino_lib::{
    execution::Axis,
    supportive_data::FractionalCoordRange,
    SearchJob,
};
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct TaskTable<'a> {
    model_content: &'a str,
    element_symbol: String,
    target_bondlength: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64, // pub(crate) export_dir: String:f64,
                // pub(crate) potential_dir: Option<String>,
                // pub(crate) edft: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn create_task_table<'a>(
    model_content: &'a str,
    element_symbol: String,
    target_bondlength: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64,
) -> TaskTable<'a> {
    TaskTable {
        model_content,
        element_symbol,
        target_bondlength,
        x_min,
        x_max,
        y_min,
        y_max,
        z_min,
        z_max,
    }
}

impl<'a> TaskTable<'a> {
    pub fn model_content(&self) -> &str {
        self.model_content
    }

    pub fn element_symbol(&self) -> &Element {
        ELEMENT_TABLE.get_by_symbol(
            ElementSymbol::from_str(&self.element_symbol)
                .expect("element symbol must be a valid element"),
        )
    }

    pub fn target_bondlength(&self) -> f64 {
        self.target_bondlength
    }

    // pub fn export_dir(&self) -> &str {
    //     &self.export_dir
    // }

    // pub fn kpoint_quality(&self) -> &KPointQuality {
    //     &self.kpoint_quality
    // }

    // pub fn edft(&self) -> bool {
    //     self.edft
    // }

    // pub fn potential_dir(&self) -> Option<&String> {
    //     &self.potential_dir
    // }
    pub fn x_range(&self) -> FractionalCoordRange {
        FractionalCoordRange::new(self.x_min, self.x_max)
    }
    pub fn y_range(&self) -> FractionalCoordRange {
        FractionalCoordRange::new(self.y_min, self.y_max)
    }
    pub fn z_range(&self) -> FractionalCoordRange {
        FractionalCoordRange::new(self.z_min, self.z_max)
    }
}

impl SearchJob for TaskTable<'_> {
    fn target_bondlength(&self) -> f64 {
        self.target_bondlength
    }

    fn search_range(&self, axis: Axis) -> FractionalCoordRange {
        match axis {
            Axis::X => self.x_range(),
            Axis::Y => self.y_range(),
            Axis::Z => self.z_range(),
        }
    }
}
