use crate::common::core::VtkIdType;
use crate::common::data_model::{Atom, Molecule};

/// VTK: `vtkBond`.
#[derive(Clone, Debug, PartialEq)]
pub struct Bond {
    molecule: Molecule,
    id: VtkIdType,
    begin_atom_id: VtkIdType,
    end_atom_id: VtkIdType,
}

impl Bond {
    /// VTK: `vtkBond::vtkBond`.
    pub(crate) fn new(
        molecule: Molecule,
        id: VtkIdType,
        begin_atom_id: VtkIdType,
        end_atom_id: VtkIdType,
    ) -> Self {
        assert!(id < molecule.get_number_of_bonds());
        assert!(begin_atom_id < molecule.get_number_of_atoms());
        assert!(end_atom_id < molecule.get_number_of_atoms());
        Self {
            molecule,
            id,
            begin_atom_id,
            end_atom_id,
        }
    }

    /// VTK: `vtkBond::PrintSelf`.
    pub fn print_self(&self) -> String {
        format!(
            "Molecule: {:?} Id: {} Order: {} Length: {} BeginAtomId: {} EndAtomId: {}\n",
            self.get_molecule(),
            self.id,
            self.get_order(),
            self.get_length(),
            self.begin_atom_id,
            self.end_atom_id
        )
    }

    /// VTK: `vtkBond::GetId`.
    pub fn get_id(&self) -> VtkIdType {
        self.id
    }

    /// VTK: `vtkBond::GetMolecule`.
    pub fn get_molecule(&self) -> Molecule {
        self.molecule.clone()
    }

    /// VTK: `vtkBond::GetBeginAtomId`.
    pub fn get_begin_atom_id(&self) -> VtkIdType {
        self.begin_atom_id
    }

    /// VTK: `vtkBond::GetEndAtomId`.
    pub fn get_end_atom_id(&self) -> VtkIdType {
        self.end_atom_id
    }

    /// VTK: `vtkBond::GetBeginAtom`.
    pub fn get_begin_atom(&self) -> Atom {
        self.molecule.get_atom(self.begin_atom_id)
    }

    /// VTK: `vtkBond::GetEndAtom`.
    pub fn get_end_atom(&self) -> Atom {
        self.molecule.get_atom(self.end_atom_id)
    }

    /// VTK: `vtkBond::GetOrder`.
    pub fn get_order(&self) -> u16 {
        self.molecule.get_bond_order(self.id)
    }

    /// VTK: `vtkBond::GetLength`.
    pub fn get_length(&self) -> f64 {
        let begin = self.molecule.get_atom_position(self.begin_atom_id);
        let end = self.molecule.get_atom_position(self.end_atom_id);
        let dx = begin[0] - end[0];
        let dy = begin[1] - end[1];
        let dz = begin[2] - end[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
