use crate::common::core::VtkIdType;

use super::Molecule;

/// VTK: `vtkAtom`.
#[derive(Clone, Debug, PartialEq)]
pub struct Atom {
    molecule: Molecule,
    id: VtkIdType,
}

impl Atom {
    /// VTK: `vtkAtom::vtkAtom`.
    pub(crate) fn new(molecule: Molecule, id: VtkIdType) -> Self {
        assert!(id < molecule.get_number_of_atoms());
        Self { molecule, id }
    }

    /// VTK: `vtkAtom::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Molecule: {:?} Id: {} Element: {} Position: {:?}\n",
            self.get_molecule(),
            self.id,
            self.get_atomic_number(),
            self.get_position()
        )
    }

    /// VTK: `vtkAtom::GetId`.
    pub fn get_id(&self) -> VtkIdType {
        self.id
    }

    /// VTK: `vtkAtom::GetMolecule`.
    pub fn get_molecule(&self) -> Molecule {
        self.molecule.clone()
    }

    /// VTK: `vtkAtom::GetAtomicNumber`.
    pub fn get_atomic_number(&self) -> u16 {
        self.molecule.get_atom_atomic_number(self.id)
    }

    /// VTK: `vtkAtom::SetAtomicNumber`.
    pub fn set_atomic_number(&mut self, atomic_number: u16) {
        self.molecule.set_atom_atomic_number(self.id, atomic_number);
    }

    /// VTK: `vtkAtom::GetPosition`.
    pub fn get_position(&self) -> [f64; 3] {
        self.molecule.get_atom_position(self.id)
    }

    /// VTK: `vtkAtom::SetPosition(float x, float y, float z)`.
    pub fn set_position(&mut self, x: f64, y: f64, z: f64) {
        self.set_position_array([x, y, z]);
    }

    /// VTK: `vtkAtom::SetPosition(const float pos[3])`.
    pub fn set_position_array(&mut self, position: [f64; 3]) {
        self.molecule.set_atom_position(self.id, position);
    }
}
