use crate::data::{DataSetAttributes, FieldData, Points};

/// An atom in a molecule.
#[derive(Debug, Clone)]
pub struct Atom {
    /// Atomic number (1=H, 6=C, 7=N, 8=O, etc.).
    pub atomic_number: u16,
    /// 3D position.
    pub position: [f64; 3],
}

/// A bond between two atoms.
#[derive(Debug, Clone, Copy)]
pub struct Bond {
    /// Index of the first atom.
    pub atom1: usize,
    /// Index of the second atom.
    pub atom2: usize,
    /// Bond order (1=single, 2=double, 3=triple).
    pub order: u16,
}

/// A molecular data structure with atoms and bonds.
///
/// Analogous to VTK's `vtkMolecule`. Stores atomic positions, atomic numbers,
/// and bond connectivity. Supports per-atom and per-bond data arrays.
#[derive(Debug, Clone)]
pub struct Molecule {
    atoms: Vec<Atom>,
    bonds: Vec<Bond>,
    /// Unit-cell lattice vectors. `None` means no unit-cell lattice is defined.
    lattice: Option<[[f64; 3]; 3]>,
    /// Unit-cell origin for rendering.
    lattice_origin: [f64; 3],
    /// Per-atom data attributes.
    atom_data: DataSetAttributes,
    /// Per-bond data attributes.
    bond_data: DataSetAttributes,
    /// Field data (metadata).
    field_data: FieldData,
}

impl Molecule {
    /// Create an empty molecule.
    pub fn new() -> Self {
        Self {
            atoms: Vec::new(),
            bonds: Vec::new(),
            lattice: None,
            lattice_origin: [0.0, 0.0, 0.0],
            atom_data: DataSetAttributes::new(),
            bond_data: DataSetAttributes::new(),
            field_data: FieldData::new(),
        }
    }

    /// Add an atom and return its index.
    pub fn add_atom(&mut self, atomic_number: u16, position: [f64; 3]) -> usize {
        let idx = self.atoms.len();
        self.atoms.push(Atom {
            atomic_number,
            position,
        });
        idx
    }

    /// Add an atom, returning its index.
    pub fn try_add_atom(&mut self, atomic_number: u16, position: [f64; 3]) -> Option<usize> {
        Some(self.add_atom(atomic_number, position))
    }

    /// Add a bond between two atoms. Returns the bond index.
    pub fn add_bond(&mut self, atom1: usize, atom2: usize, order: u16) -> usize {
        self.try_add_bond(atom1, atom2, order)
            .expect("bond atom index out of bounds for Molecule")
    }

    /// Add a bond between two atoms, returning `None` if either atom index is invalid.
    pub fn try_add_bond(&mut self, atom1: usize, atom2: usize, order: u16) -> Option<usize> {
        if atom1 >= self.atoms.len() || atom2 >= self.atoms.len() {
            return None;
        }
        let idx = self.bonds.len();
        self.bonds.push(Bond {
            atom1,
            atom2,
            order,
        });
        Some(idx)
    }

    /// Number of atoms.
    pub fn num_atoms(&self) -> usize {
        self.atoms.len()
    }

    /// Number of bonds.
    pub fn num_bonds(&self) -> usize {
        self.bonds.len()
    }

    /// Get atom by index.
    pub fn atom(&self, idx: usize) -> &Atom {
        &self.atoms[idx]
    }

    /// Get bond by index.
    pub fn bond(&self, idx: usize) -> &Bond {
        &self.bonds[idx]
    }

    /// Iterate over atoms.
    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }

    /// Iterate over bonds.
    pub fn bonds(&self) -> &[Bond] {
        &self.bonds
    }

    /// Get atom positions as Points.
    pub fn positions(&self) -> Points<f64> {
        let mut pts = Points::new();
        for atom in &self.atoms {
            pts.push(atom.position);
        }
        pts
    }

    /// Return true if a unit-cell lattice is defined.
    pub fn has_lattice(&self) -> bool {
        self.lattice.is_some()
    }

    /// Remove any unit-cell lattice information and reset the lattice origin.
    pub fn clear_lattice(&mut self) {
        self.lattice = None;
        self.lattice_origin = [0.0, 0.0, 0.0];
    }

    /// Set the unit-cell lattice vectors.
    pub fn set_lattice(&mut self, a: [f64; 3], b: [f64; 3], c: [f64; 3]) {
        self.lattice = Some([a, b, c]);
    }

    /// Get the unit-cell lattice vectors. Returns zero vectors if none are set.
    pub fn lattice(&self) -> [[f64; 3]; 3] {
        self.lattice.unwrap_or([[0.0; 3]; 3])
    }

    /// Unit-cell lattice vectors, if present.
    pub fn lattice_opt(&self) -> Option<[[f64; 3]; 3]> {
        self.lattice
    }

    /// Set the unit-cell origin.
    pub fn set_lattice_origin(&mut self, origin: [f64; 3]) {
        self.lattice_origin = origin;
    }

    /// Get the unit-cell origin.
    pub fn lattice_origin(&self) -> [f64; 3] {
        self.lattice_origin
    }

    /// Set periodic unit-cell vectors for crystalline or periodic molecule data.
    pub fn set_periodic_cell(&mut self, cell: [[f64; 3]; 3]) -> bool {
        self.set_lattice(cell[0], cell[1], cell[2]);
        true
    }

    /// Clear periodic unit-cell vectors.
    pub fn clear_periodic_cell(&mut self) {
        self.clear_lattice();
    }

    /// Periodic unit-cell vectors, if present.
    pub fn periodic_cell(&self) -> Option<[[f64; 3]; 3]> {
        self.lattice_opt()
    }

    /// Whether this molecule has periodic unit-cell metadata.
    pub fn is_periodic(&self) -> bool {
        self.has_lattice()
    }

    /// Element symbol for an atomic number.
    pub fn element_symbol(atomic_number: u16) -> &'static str {
        const SYMBOLS: [&str; 119] = [
            "Xx", "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si",
            "P", "S", "Cl", "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu",
            "Zn", "Ga", "Ge", "As", "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc",
            "Ru", "Rh", "Pd", "Ag", "Cd", "In", "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La",
            "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb", "Dy", "Ho", "Er", "Tm", "Yb", "Lu",
            "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl", "Pb", "Bi", "Po", "At",
            "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk", "Cf", "Es",
            "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Uut",
            "Fl", "Uup", "Lv", "Uus", "Uuo",
        ];
        SYMBOLS
            .get(atomic_number as usize)
            .copied()
            .unwrap_or(SYMBOLS[0])
    }

    /// Approximate covalent radius in Angstroms for common elements.
    pub fn covalent_radius(atomic_number: u16) -> f64 {
        match atomic_number {
            1 => 0.31,
            6 => 0.76,
            7 => 0.71,
            8 => 0.66,
            9 => 0.57,
            15 => 1.07,
            16 => 1.05,
            17 => 1.02,
            35 => 1.20,
            53 => 1.39,
            _ => 0.8,
        }
    }

    /// CPK color for common elements (RGB 0-1).
    pub fn cpk_color(atomic_number: u16) -> [f32; 3] {
        match atomic_number {
            1 => [1.0, 1.0, 1.0],  // H: white
            6 => [0.2, 0.2, 0.2],  // C: dark gray
            7 => [0.0, 0.0, 1.0],  // N: blue
            8 => [1.0, 0.0, 0.0],  // O: red
            9 => [0.0, 1.0, 0.0],  // F: green
            15 => [1.0, 0.5, 0.0], // P: orange
            16 => [1.0, 1.0, 0.0], // S: yellow
            17 => [0.0, 1.0, 0.0], // Cl: green
            26 => [0.5, 0.3, 0.0], // Fe: brown
            _ => [0.8, 0.5, 1.0],  // default: pink
        }
    }

    pub fn atom_data(&self) -> &DataSetAttributes {
        &self.atom_data
    }

    pub fn atom_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.atom_data
    }

    pub fn bond_data(&self) -> &DataSetAttributes {
        &self.bond_data
    }

    pub fn bond_data_mut(&mut self) -> &mut DataSetAttributes {
        &mut self.bond_data
    }

    pub fn field_data(&self) -> &FieldData {
        &self.field_data
    }

    pub fn field_data_mut(&mut self) -> &mut FieldData {
        &mut self.field_data
    }
}

impl Default for Molecule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_molecule() {
        let mut mol = Molecule::new();
        let o = mol.add_atom(8, [0.0, 0.0, 0.0]);
        let h1 = mol.add_atom(1, [0.757, 0.586, 0.0]);
        let h2 = mol.add_atom(1, [-0.757, 0.586, 0.0]);
        mol.add_bond(o, h1, 1);
        mol.add_bond(o, h2, 1);

        assert_eq!(mol.num_atoms(), 3);
        assert_eq!(mol.num_bonds(), 2);
        assert_eq!(mol.atom(0).atomic_number, 8);
        assert_eq!(Molecule::element_symbol(8), "O");
        assert_eq!(Molecule::element_symbol(118), "Uuo");
        assert_eq!(Molecule::element_symbol(119), "Xx");
    }

    #[test]
    fn methane_molecule() {
        let mut mol = Molecule::new();
        let c = mol.add_atom(6, [0.0, 0.0, 0.0]);
        for pos in &[
            [0.63, 0.63, 0.63],
            [-0.63, -0.63, 0.63],
            [-0.63, 0.63, -0.63],
            [0.63, -0.63, -0.63],
        ] {
            let h = mol.add_atom(1, *pos);
            mol.add_bond(c, h, 1);
        }

        assert_eq!(mol.num_atoms(), 5);
        assert_eq!(mol.num_bonds(), 4);
    }

    #[test]
    fn positions() {
        let mut mol = Molecule::new();
        mol.add_atom(6, [1.0, 2.0, 3.0]);
        mol.add_atom(8, [4.0, 5.0, 6.0]);

        let pts = mol.positions();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts.get(0), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn cpk_colors() {
        let c = Molecule::cpk_color(6);
        assert_eq!(c, [0.2, 0.2, 0.2]);
        let o = Molecule::cpk_color(8);
        assert_eq!(o, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn covalent_radii() {
        assert!(Molecule::covalent_radius(1) < Molecule::covalent_radius(6));
    }

    #[test]
    fn try_add_bond_rejects_invalid_atom_indices() {
        let mut mol = Molecule::new();
        let c = mol.add_atom(6, [0.0, 0.0, 0.0]);
        assert_eq!(mol.try_add_bond(c, 10, 1), None);
        assert_eq!(mol.num_bonds(), 0);
    }

    #[test]
    fn atom_positions_and_lattice_follow_vtk_storage_rules() {
        let mut mol = Molecule::new();
        assert_eq!(mol.try_add_atom(6, [f64::NAN, 0.0, 0.0]), Some(0));
        assert_eq!(mol.num_atoms(), 1);

        assert!(mol.set_periodic_cell([[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]]));
        assert!(mol.is_periodic());
        mol.set_lattice_origin([4.0, 5.0, 6.0]);
        assert_eq!(mol.periodic_cell().unwrap()[1][1], 2.0);
        assert_eq!(mol.lattice_origin(), [4.0, 5.0, 6.0]);
        mol.clear_lattice();
        assert!(!mol.has_lattice());
        assert_eq!(mol.lattice(), [[0.0; 3]; 3]);
        assert_eq!(mol.lattice_origin(), [0.0, 0.0, 0.0]);
    }
}
