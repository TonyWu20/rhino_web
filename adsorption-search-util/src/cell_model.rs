//! Extracts the [`chemrust_core::data::lattice::CrystalModel`] data from a
//! 0.7.0 [`castep_cell_io::CellDocument`].
//!
//! `castep-cell-io` 0.7.0 no longer depends on `chemrust-core`, so
//! `CellDocument` no longer implements the `CrystalModel` trait. The
//! adapter below reconstructs the cell parameters and atom data that the
//! `rhino` search flow needs.

use std::convert::TryInto;

use castep_cell_io::cell::species::Species;
use castep_cell_io::{CellDocument, Lattice, Positions};
use castep_periodic_table::element::ElementSymbol;
use chemrust_core::data::{
    atom::CoreAtomData,
    geom::coordinates::CoordData,
    lattice::{CellConstants, CrystalModel, LatticeVectors, UnitCellParameters},
};
use nalgebra::{Matrix3, Point3, RowVector3};

/// Cell parameters extracted from a [`CellDocument`].
enum CellParamRepr {
    Vectors(LatticeVectors),
    Constants(CellConstants),
}

impl UnitCellParameters for CellParamRepr {
    fn lattice_bases(&self) -> Matrix3<f64> {
        match self {
            Self::Vectors(v) => v.lattice_bases(),
            Self::Constants(c) => c.lattice_bases(),
        }
    }
}

/// Atom data extracted from a [`CellDocument`].
struct AdapterAtomData {
    indices: Vec<usize>,
    symbols: Vec<ElementSymbol>,
    coords: Vec<CoordData>,
    labels: Vec<Option<String>>,
}

impl CoreAtomData for AdapterAtomData {
    fn indices_repr(&self) -> Vec<usize> {
        self.indices.clone()
    }

    fn symbols_repr(&self) -> Vec<ElementSymbol> {
        self.symbols.clone()
    }

    fn coords_repr(&self) -> Vec<CoordData> {
        self.coords.clone()
    }

    fn labels_repr(&self) -> Vec<Option<String>> {
        self.labels.clone()
    }
}

/// Owns the extracted cell parameters and atom data of a 0.7.0
/// [`CellDocument`] and exposes them through the [`CrystalModel`] trait.
pub struct CellModelAdapter {
    cell_params: CellParamRepr,
    atom_data: AdapterAtomData,
}

impl CellModelAdapter {
    /// Extracts the lattice and position data of `doc`.
    pub fn new(doc: &CellDocument) -> Self {
        let cell_params = lattice_repr(&doc.lattice);
        let atom_data = atom_data_from_positions(&doc.positions);
        Self {
            cell_params,
            atom_data,
        }
    }
}

impl CrystalModel for CellModelAdapter {
    fn get_cell_parameters(&self) -> &impl UnitCellParameters {
        &self.cell_params
    }

    fn get_atom_data(&self) -> &impl CoreAtomData {
        &self.atom_data
    }

    fn get_cell_parameters_mut(&mut self) -> &mut impl UnitCellParameters {
        &mut self.cell_params
    }

    fn get_atom_data_mut(&mut self) -> &mut impl CoreAtomData {
        &mut self.atom_data
    }
}

/// Builds the cell parameter representation of a 0.7.0 [`Lattice`].
///
/// `LatticeCart` stores the lattice vectors as rows; `chemrust-core`
/// expects them as columns of `lattice_bases()`. `LatticeABC` angles are
/// degrees in the file and must be radians in [`CellConstants`].
fn lattice_repr(lattice: &Lattice) -> CellParamRepr {
    match lattice {
        Lattice::Cart(cart) => {
            let m = Matrix3::from_rows(&[
                RowVector3::from(cart.a),
                RowVector3::from(cart.b),
                RowVector3::from(cart.c),
            ])
            .transpose();
            CellParamRepr::Vectors(LatticeVectors::new(m))
        }
        Lattice::Abc(abc) => CellParamRepr::Constants(CellConstants::new(
            abc.abc[0],
            abc.abc[1],
            abc.abc[2],
            abc.angles[0].to_radians(),
            abc.angles[1].to_radians(),
            abc.angles[2].to_radians(),
        )),
    }
}

/// Maps a 0.7.0 [`Species`] value to an [`ElementSymbol`].
fn species_symbol(species: &Species) -> Option<ElementSymbol> {
    match species {
        Species::Symbol(s) => s.parse().ok(),
        Species::AtomicNumber(n) => (*n).try_into().ok(),
    }
}

fn atom_data_from_positions(positions: &Positions) -> AdapterAtomData {
    let (symbols, coords): (Vec<ElementSymbol>, Vec<CoordData>) = match positions {
        Positions::Frac(frac) => (
            frac.positions
                .iter()
                .map(|e| species_symbol(&e.species).expect("species must map to a known element"))
                .collect(),
            frac.positions
                .iter()
                .map(|e| {
                    CoordData::Fractional(Point3::new(e.coord[0], e.coord[1], e.coord[2]))
                })
                .collect(),
        ),
        Positions::Abs(abs) => (
            abs.positions
                .iter()
                .map(|e| species_symbol(&e.species).expect("species must map to a known element"))
                .collect(),
            abs.positions
                .iter()
                .map(|e| {
                    CoordData::Cartesian(Point3::new(e.coord[0], e.coord[1], e.coord[2]))
                })
                .collect(),
        ),
    };
    let n = coords.len();
    AdapterAtomData {
        indices: (0..n).collect(),
        symbols,
        coords,
        labels: vec![None; n],
    }
}
