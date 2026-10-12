//! Bridges between `castep-cell-io` 0.7.0 documents and the
//! `chemrust-core` [`CrystalModel`] trait used by the `rhino` search flow.
//!
//! `castep-cell-io` 0.7.0 no longer depends on `chemrust-core`, so
//! [`CellDocument`] no longer implements [`CrystalModel`]. This module
//! provides the adapter, plus helpers that build 0.7.0 export documents
//! and collect pseudopotential file names from search results.

use std::{collections::{HashMap, HashSet}, fs, path::Path};

use castep_cell_fmt::Error as CError;
use castep_cell_io::cell::lattice_param::LatticeCart;
use castep_cell_io::cell::positions::{PositionFracEntry, PositionsFrac};
use castep_cell_io::cell::species::Species;
use castep_cell_io::{CellDocument, Lattice, Positions};
use castep_periodic_table::{
    data::ELEMENT_TABLE,
    element::{ElementSymbol, LookupElement},
};
use chemrust_core::data::{
    atom::CoreAtomData,
    geom::coordinates::CoordData,
    lattice::{CellConstants, CrystalModel, LatticeVectors, UnitCellParameters},
};
use nalgebra::{Matrix3, Point3, RowVector3, Vector3};

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

/// Builds the [`LatticeVectors`] / [`CellConstants`] representation of a
/// 0.7.0 [`Lattice`].
///
/// `LatticeCart` stores the lattice vectors as rows; `chemrust-core`
/// expects them as columns of `lattice_bases()`. `LatticeABC` angles are
/// degrees in the file and must be converted to radians for
/// [`CellConstants`].
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

/// Computes the column-convention lattice basis matrix of `lattice`.
pub fn lattice_bases(lattice: &Lattice) -> Matrix3<f64> {
    lattice_repr(lattice).lattice_bases()
}

/// Maps a 0.7.0 [`Species`] value to an [`ElementSymbol`].
fn species_symbol(species: &Species) -> Option<ElementSymbol> {
    match species {
        Species::Symbol(s) => s.parse().ok(),
        Species::AtomicNumber(n) => (*n).try_into().ok(),
    }
}

/// Expects a valid [`ElementSymbol`] for `species` and panics otherwise.
fn species_symbol_expect(species: &Species) -> ElementSymbol {
    species_symbol(species).unwrap_or_else(|| {
        panic!(
            "species {:?} must map to a known element symbol",
            species
        )
    })
}

fn symbol_to_species(symbol: &ElementSymbol) -> Species {
    Species::Symbol(symbol.to_string())
}

fn atom_data_from_positions(positions: &Positions) -> AdapterAtomData {
    let (symbols, coords): (Vec<ElementSymbol>, Vec<CoordData>) = match positions {
        Positions::Frac(frac) => (
            frac.positions
                .iter()
                .map(|e| species_symbol_expect(&e.species))
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
                .map(|e| species_symbol_expect(&e.species))
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

/// Clones `doc` and appends the fractional coordinate of `site` to its
/// positions. Absolute positions are converted to fractional so that the
/// resulting document carries a single coordinate representation.
pub fn add_site_to_cell_document(
    doc: &CellDocument,
    site: &impl chemrust_nasl::Visualize,
    element: &ElementSymbol,
) -> CellDocument {
    let mut doc = doc.clone();
    let bases = lattice_bases(&doc.lattice);
    let frac = site.fractional_coord(bases);
    let entry = PositionFracEntry {
        species: symbol_to_species(element),
        coord: [frac.x, frac.y, frac.z],
        spin: None,
        mixture: None,
    };
    match doc.positions {
        Positions::Frac(mut frac_positions) => {
            frac_positions.positions.push(entry);
            doc.positions = Positions::Frac(frac_positions);
        }
        Positions::Abs(abs_positions) => {
            let inv = bases
                .try_inverse()
                .expect("lattice vectors must be invertible");
            let mut entries: Vec<PositionFracEntry> = abs_positions
                .positions
                .iter()
                .map(|e| {
                    let cart = Vector3::new(e.coord[0], e.coord[1], e.coord[2]);
                    let frac = inv * cart;
                    PositionFracEntry {
                        species: e.species.clone(),
                        coord: [frac.x, frac.y, frac.z],
                        spin: e.spin,
                        mixture: e.mixture,
                    }
                })
                .collect();
            entries.push(entry);
            doc.positions = Positions::Frac(PositionsFrac { positions: entries });
        }
    }
    doc
}

/// Builds a new 0.7.0 [`CellDocument`] from an arbitrary [`CrystalModel`]
/// (for example a parsed CIF data block) plus one new site.
///
/// The new document carries a `LATTICE_CART` block and fractional
/// positions. It does not carry the optional blocks of a parsed cell
/// document; use [`add_site_to_cell_document`] for that.
pub fn cell_document_from_model(
    model: &impl CrystalModel,
    element: &ElementSymbol,
    site: &impl chemrust_nasl::Visualize,
) -> Result<CellDocument, CError> {
    let bases = model.get_cell_parameters().lattice_bases();
    let inv = bases
        .try_inverse()
        .ok_or_else(|| CError::Message("lattice vectors must be invertible".to_string()))?;
    let atoms = model.get_atom_data();
    let mut positions: Vec<PositionFracEntry> = atoms
        .coords_repr()
        .iter()
        .enumerate()
        .map(|(i, cd)| {
            let frac = match cd {
                CoordData::Fractional(p) => *p,
                CoordData::Cartesian(p) => inv * *p,
            };
            PositionFracEntry {
                species: symbol_to_species(&atoms.symbols_repr()[i]),
                coord: [frac.x, frac.y, frac.z],
                spin: None,
                mixture: None,
            }
        })
        .collect();
    let site_frac = inv * site.determine_coord();
    positions.push(PositionFracEntry {
        species: symbol_to_species(element),
        coord: [site_frac.x, site_frac.y, site_frac.z],
        spin: None,
        mixture: None,
    });
    let cart = LatticeCart {
        unit: None,
        a: bases.column(0).into(),
        b: bases.column(1).into(),
        c: bases.column(2).into(),
    };
    CellDocument::builder()
        .lattice(Lattice::Cart(cart))
        .positions(Positions::Frac(PositionsFrac { positions }))
        .build()
}

/// Collects the pseudopotential file names of `docs`.
///
/// A document that carries a `SPECIES_POT` block contributes its explicit
/// entries. A document without that block contributes the default
/// potential of every element present in its positions.
pub fn potential_names(docs: &[&CellDocument]) -> HashSet<String> {
    let mut names = HashSet::new();
    for doc in docs {
        if let Some(sp) = &doc.species.species_pot {
            names.extend(sp.potentials.iter().map(|e| e.filename.clone()));
        } else {
            for species in position_species(doc) {
                if let Some(symbol) = species_symbol(&species) {
                    names.insert(ELEMENT_TABLE.get_by_symbol(symbol).potential().to_string());
                }
            }
        }
    }
    names
}

/// Reads the files named in `names` under `potential_loc`.
pub fn read_potentials(
    names: &HashSet<String>,
    potential_loc: &Path,
) -> std::io::Result<HashMap<String, Vec<u8>>> {
    names
        .iter()
        .map(|name| {
            let bytes = fs::read(potential_loc.join(name))?;
            Ok((name.clone(), bytes))
        })
        .collect()
}

fn position_species(doc: &CellDocument) -> Vec<Species> {
    match &doc.positions {
        Positions::Frac(frac) => frac
            .positions
            .iter()
            .map(|e| e.species.clone())
            .collect(),
        Positions::Abs(abs) => abs
            .positions
            .iter()
            .map(|e| e.species.clone())
            .collect(),
    }
}
