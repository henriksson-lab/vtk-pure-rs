use crate::common::core::VtkIdType;
use crate::common::{
    core::{points::Points, AnyArray, UnsignedShortArray},
    data_model::{Atom, Bond, FieldDataArray, MutableUndirectedGraph},
};

use std::{cell::UnsafeCell, rc::Rc};

#[derive(Clone, Debug, PartialEq)]
struct MoleculeData {
    graph: MutableUndirectedGraph,
    lattice: Option<[[f64; 3]; 3]>,
    lattice_origin: [f64; 3],
}

#[derive(Debug)]
struct MoleculeStorage {
    data: UnsafeCell<MoleculeData>,
}

impl MoleculeStorage {
    fn new(data: MoleculeData) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    fn data(&self) -> &MoleculeData {
        unsafe { &*self.data.get() }
    }

    fn data_mut(&self) -> &mut MoleculeData {
        unsafe { &mut *self.data.get() }
    }
}

/// Molecular geometry and connectivity.
///
/// VTK origin: selected audited symbols from
/// `VTK/Common/DataModel/vtkMolecule.cxx` and `vtkMolecule.h`.
#[derive(Debug)]
pub struct Molecule {
    storage: Rc<MoleculeStorage>,
}

impl Clone for Molecule {
    fn clone(&self) -> Self {
        Self {
            storage: Rc::clone(&self.storage),
        }
    }
}

impl PartialEq for Molecule {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.storage, &other.storage) || self.storage.data() == other.storage.data()
    }
}

impl Molecule {
    /// VTK: `vtkMolecule::New`.
    pub fn new() -> Self {
        Self {
            storage: Rc::new(MoleculeStorage::new(empty_storage())),
        }
    }

    fn storage(&self) -> &MoleculeData {
        self.storage.data()
    }

    fn storage_mut(&self) -> &mut MoleculeData {
        self.storage.data_mut()
    }

    /// VTK: `vtkMolecule::AppendAtom`.
    pub fn append_atom(&mut self, atomic_number: u16, x: f64, y: f64, z: f64) -> Atom {
        self.append_atom_at(atomic_number, [x, y, z])
    }

    fn append_atom_at(&mut self, atomic_number: u16, position: [f64; 3]) -> Atom {
        let id = {
            let storage = self.storage_mut();
            let id = storage.graph.add_vertex();
            storage.graph.get_points_mut().set_point(id, position);
            atomic_numbers_array_mut(&mut storage.graph).set_typed_tuple(id, &[atomic_number]);
            id
        };
        Atom::new(self.clone(), id)
    }

    /// VTK: `vtkMolecule::GetNumberOfAtoms`.
    pub fn get_number_of_atoms(&self) -> VtkIdType {
        self.storage().graph.get_number_of_vertices()
    }

    /// VTK: `vtkMolecule::GetAtom`.
    pub fn get_atom(&self, atom_id: VtkIdType) -> Atom {
        Atom::new(self.clone(), atom_id)
    }

    /// VTK: `vtkMolecule::GetAtomAtomicNumber`.
    pub fn get_atom_atomic_number(&self, atom_id: VtkIdType) -> u16 {
        self.get_atomic_number_array().as_slice()[vtk_id_to_index(atom_id)]
    }

    /// VTK: `vtkMolecule::SetAtomAtomicNumber`.
    pub fn set_atom_atomic_number(&mut self, atom_id: VtkIdType, atomic_number: u16) {
        atomic_numbers_array_mut(&mut self.storage_mut().graph)
            .set_typed_tuple(atom_id, &[atomic_number]);
    }

    /// VTK: `vtkMolecule::GetAtomPosition`.
    pub fn get_atom_position(&self, atom_id: VtkIdType) -> [f64; 3] {
        self.storage()
            .graph
            .points_ref()
            .expect("vtkMolecule atomic positions exist")
            .get_point(atom_id)
    }

    /// VTK: `vtkMolecule::SetAtomPosition`.
    pub fn set_atom_position(&mut self, atom_id: VtkIdType, position: [f64; 3]) {
        self.storage_mut()
            .graph
            .get_points_mut()
            .set_point(atom_id, position);
    }

    /// VTK: `vtkMolecule::AppendBond`.
    pub fn append_bond(&mut self, atom1: VtkIdType, atom2: VtkIdType, order: u16) -> Bond {
        let id = {
            let storage = self.storage_mut();
            let edge = storage.graph.add_edge(atom1, atom2);
            let id = edge.id;
            bond_orders_array_mut(&mut storage.graph).set_typed_tuple(id, &[order]);
            id
        };
        Bond::new(self.clone(), id, atom1, atom2)
    }

    /// VTK: `vtkMolecule::GetNumberOfBonds`.
    pub fn get_number_of_bonds(&self) -> VtkIdType {
        self.storage().graph.get_number_of_edges()
    }

    /// VTK: `vtkMolecule::GetBond`.
    pub fn get_bond(&self, bond_id: VtkIdType) -> Bond {
        let graph = self.storage().graph.as_graph();
        let source = graph.get_source_vertex(bond_id);
        let target = graph.get_target_vertex(bond_id);
        assert!(
            source >= 0 && target >= 0,
            "bond id out of bounds for Molecule"
        );
        Bond::new(self.clone(), bond_id, source, target)
    }

    /// VTK: `vtkMolecule::GetBondOrder`.
    pub fn get_bond_order(&self, bond_id: VtkIdType) -> u16 {
        self.get_bond_orders_array().as_slice()[vtk_id_to_index(bond_id)]
    }

    /// VTK: `vtkMolecule::SetBondOrder`.
    pub fn set_bond_order(&mut self, bond_id: VtkIdType, order: u16) {
        bond_orders_array_mut(&mut self.storage_mut().graph).set_typed_tuple(bond_id, &[order]);
    }

    /// VTK: `vtkMolecule::GetBondLength`.
    pub fn get_bond_length(&self, bond_id: VtkIdType) -> f64 {
        let bond = self.get_bond(bond_id);
        bond.get_length()
    }

    /// VTK: `vtkMolecule::HasLattice`.
    pub fn has_lattice(&self) -> bool {
        self.storage().lattice.is_some()
    }

    /// VTK: `vtkMolecule::ClearLattice`.
    pub fn clear_lattice(&mut self) {
        let storage = self.storage_mut();
        if storage.lattice.is_some() {
            storage.lattice = None;
            storage.lattice_origin = [0.0, 0.0, 0.0];
        }
    }

    /// VTK: `vtkMolecule::SetLattice(const vtkVector3d&, ...)`.
    pub fn set_lattice(&mut self, a: [f64; 3], b: [f64; 3], c: [f64; 3]) {
        self.storage_mut().lattice = Some([a, b, c]);
    }

    /// VTK: `vtkMolecule::GetLattice(vtkVector3d&, ...)`.
    pub fn get_lattice(&self) -> [[f64; 3]; 3] {
        self.storage().lattice.unwrap_or([[0.0; 3]; 3])
    }

    /// VTK: `vtkMolecule::SetLatticeOrigin`.
    pub fn set_lattice_origin(&mut self, origin: [f64; 3]) {
        self.storage_mut().lattice_origin = origin;
    }

    /// VTK: `vtkMolecule::GetLatticeOrigin`.
    pub fn get_lattice_origin(&self) -> [f64; 3] {
        self.storage().lattice_origin
    }

    /// VTK: `vtkMolecule::DeepCopy`.
    pub fn deep_copy(&mut self, source: &Self) {
        self.storage = Rc::new(MoleculeStorage::new(source.storage().clone()));
    }

    /// VTK: `vtkMolecule::ShallowCopy`.
    pub fn shallow_copy(&mut self, source: &Self) {
        self.storage = Rc::clone(&source.storage);
    }

    /// VTK: `vtkMolecule::GetAtomicPositionArray`.
    pub fn get_atomic_position_array(&self) -> &Points {
        self.storage()
            .graph
            .points_ref()
            .expect("vtkMolecule atomic positions exist")
    }

    /// VTK: `vtkMolecule::GetAtomicNumberArray`.
    pub fn get_atomic_number_array(&self) -> &UnsignedShortArray {
        atomic_numbers_array(&self.storage().graph)
    }

    /// VTK: `vtkMolecule::GetBondOrdersArray`.
    pub fn get_bond_orders_array(&self) -> &UnsignedShortArray {
        bond_orders_array(&self.storage().graph)
    }
}

const ATOMIC_NUMBERS_NAME: &str = "Atomic Numbers";
const BOND_ORDERS_NAME: &str = "Bond Orders";

fn empty_storage() -> MoleculeData {
    let mut graph = MutableUndirectedGraph::new();
    graph.set_points(Points::new());
    assert!(
        graph
            .get_vertex_data_mut()
            .set_field_data_scalars(Some(unsigned_short_field_array(ATOMIC_NUMBERS_NAME)))
            >= 0,
        "atomic-number scalars are one-component arrays"
    );
    assert!(
        graph
            .get_edge_data_mut()
            .set_field_data_scalars(Some(unsigned_short_field_array(BOND_ORDERS_NAME)))
            >= 0,
        "bond-order scalars are one-component arrays"
    );
    MoleculeData {
        graph,
        lattice: None,
        lattice_origin: [0.0, 0.0, 0.0],
    }
}

fn unsigned_short_field_array(name: &str) -> FieldDataArray {
    FieldDataArray::from_any_array(AnyArray::UnsignedShort(
        UnsignedShortArray::with_name_and_number_of_components(name, 1),
    ))
}

fn atomic_numbers_array(graph: &MutableUndirectedGraph) -> &UnsignedShortArray {
    match graph
        .get_vertex_data()
        .get_field_data_scalars()
        .expect("vtkMolecule atomic number array exists")
        .get_data()
    {
        AnyArray::UnsignedShort(array) => array,
        _ => panic!("vtkMolecule atomic number array is unsigned short"),
    }
}

fn atomic_numbers_array_mut(graph: &mut MutableUndirectedGraph) -> &mut UnsignedShortArray {
    match graph
        .get_vertex_data_mut()
        .get_array_mut(ATOMIC_NUMBERS_NAME)
        .expect("vtkMolecule atomic number array exists")
        .get_data_mut()
    {
        AnyArray::UnsignedShort(array) => array,
        _ => panic!("vtkMolecule atomic number array is unsigned short"),
    }
}

fn bond_orders_array(graph: &MutableUndirectedGraph) -> &UnsignedShortArray {
    match graph
        .get_edge_data()
        .get_field_data_scalars()
        .expect("vtkMolecule bond orders array exists")
        .get_data()
    {
        AnyArray::UnsignedShort(array) => array,
        _ => panic!("vtkMolecule bond orders array is unsigned short"),
    }
}

fn bond_orders_array_mut(graph: &mut MutableUndirectedGraph) -> &mut UnsignedShortArray {
    match graph
        .get_edge_data_mut()
        .get_array_mut(BOND_ORDERS_NAME)
        .expect("vtkMolecule bond orders array exists")
        .get_data_mut()
    {
        AnyArray::UnsignedShort(array) => array,
        _ => panic!("vtkMolecule bond orders array is unsigned short"),
    }
}

fn vtk_id_to_index(id: VtkIdType) -> usize {
    usize::try_from(id).expect("vtkIdType id must be non-negative and fit usize")
}
