mod error;
mod run_params;

use castep_periodic_table::element::Element;
use chemrust_core::data::{custom_data_type::FractionalCoordRange, Atom, BasicLatticeModel};
use chemrust_parser::CellParser;
use chemrust_scanner::MountingChecker;

use error::LoadError;
pub use run_params::{create_task_table, TaskTable};

#[derive(Debug, Clone, Copy)]
pub enum InputFormat {
    Msi,
    Cell,
    Cif,
}

pub fn init_checker(element_symbol: &Element, bondlength: f64) -> MountingChecker {
    MountingChecker::new_builder()
        .with_element(element_symbol)
        .with_bondlength(bondlength)
        .build()
}

pub fn search_with_task_table(table: &TaskTable) -> String {
    let mounting_checker = MountingChecker::new_builder()
        .with_element(table.element_symbol())
        .with_bondlength(table.target_bondlength())
        .build();
    let cell_model = CellParser::new(table.model_content())
        .to_lattice_cart()
        .to_positions()
        .build_lattice();
    let filtered_atoms =
        cell_model.xyz_range_filter(table.x_range(), table.y_range(), table.z_range());
    mounting_checker
        .mount_search(cell_model.atoms(), &filtered_atoms)
        .report_spheres()
}

pub fn filtered_atoms(
    model: &BasicLatticeModel,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    z_min: f64,
    z_max: f64,
) -> Vec<Atom> {
    model.xyz_range_filter(
        FractionalCoordRange::new(x_min, x_max),
        FractionalCoordRange::new(y_min, y_max),
        FractionalCoordRange::new(z_min, z_max),
    )
}

pub fn load_model(
    format_extension: &str,
    model_content: &str,
) -> Result<BasicLatticeModel, LoadError> {
    if let "cell" = format_extension {
        Ok(CellParser::new(model_content)
            .to_lattice_cart()
            .to_positions()
            .build_lattice())
    } else {
        Err(LoadError::UnsupportedFormat(format_extension.to_string()))
    }
}
