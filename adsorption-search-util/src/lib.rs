mod run_params;

use chemrust_parser::CellParser;
use chemrust_scanner::MountingChecker;

pub use run_params::{create_task_table, TaskTable};

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
        .mount_search(&filtered_atoms)
        .report_spheres()
}
