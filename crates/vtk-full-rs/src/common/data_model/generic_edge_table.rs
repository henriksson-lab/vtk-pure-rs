use crate::common::core::{Object, VtkIdType, VtkMTimeType};

use super::simple_cell_tessellator::SimpleCellTessellatorEdgeTableApi;

const DEFAULT_TABLE_SIZE: usize = 4093;

/// VTK: `vtkGenericEdgeTable`.
#[derive(Debug)]
pub struct GenericEdgeTable {
    object: Object,
    edge_table: Vec<Vec<EdgeEntry>>,
    hash_points: Vec<Vec<PointEntry>>,
    edge_modulo: VtkIdType,
    point_modulo: VtkIdType,
    last_point_id: VtkIdType,
    number_of_components: i32,
}

#[derive(Debug, Clone)]
struct PointEntry {
    point_id: VtkIdType,
    coord: [f64; 3],
    scalar: Vec<f64>,
    reference: i32,
}

impl PointEntry {
    /// VTK: `vtkGenericEdgeTable::PointEntry::PointEntry`.
    fn new(size: i32) -> Self {
        assert!(size > 0, "pre: positive_number_of_components");
        Self {
            point_id: -1,
            coord: [-100.0; 3],
            scalar: vec![0.0; size as usize],
            reference: -10,
        }
    }
}

#[derive(Debug, Clone)]
struct EdgeEntry {
    e1: VtkIdType,
    e2: VtkIdType,
    reference: i32,
    to_split: i32,
    point_id: VtkIdType,
    cell_id: VtkIdType,
}

impl Default for EdgeEntry {
    /// VTK: `vtkGenericEdgeTable::EdgeEntry::EdgeEntry`.
    fn default() -> Self {
        Self {
            e1: -1,
            e2: -1,
            reference: 0,
            to_split: 0,
            point_id: -1,
            cell_id: -1,
        }
    }
}

impl GenericEdgeTable {
    /// VTK: `vtkGenericEdgeTable::New`.
    pub fn new() -> Self {
        Self {
            object: Object::with_class_name("vtkGenericEdgeTable"),
            edge_table: vec![Vec::new(); DEFAULT_TABLE_SIZE],
            hash_points: vec![Vec::new(); DEFAULT_TABLE_SIZE],
            edge_modulo: DEFAULT_TABLE_SIZE as VtkIdType,
            point_modulo: DEFAULT_TABLE_SIZE as VtkIdType,
            last_point_id: 0,
            number_of_components: 1,
        }
    }

    /// VTK: `vtkGenericEdgeTable::PrintSelf`.
    pub fn print_self(&self) -> String {
        self.object.get_class_name().to_string()
    }

    /// VTK: `vtkGenericEdgeTable::InsertEdge`, split overload.
    pub fn insert_edge_with_point(
        &mut self,
        e1: VtkIdType,
        e2: VtkIdType,
        cell_id: VtkIdType,
        reference: i32,
        point_id: &mut VtkIdType,
    ) {
        self.insert_edge_internal(e1, e2, cell_id, reference, 1, point_id);
    }

    /// VTK: `vtkGenericEdgeTable::InsertEdge`, non-split overload.
    pub fn insert_edge(
        &mut self,
        e1: VtkIdType,
        e2: VtkIdType,
        cell_id: VtkIdType,
        reference: i32,
    ) {
        let mut point_id = -1;
        self.insert_edge_internal(e1, e2, cell_id, reference, 0, &mut point_id);
    }

    /// VTK: protected `vtkGenericEdgeTable::InsertEdge`.
    fn insert_edge_internal(
        &mut self,
        mut e1: VtkIdType,
        mut e2: VtkIdType,
        cell_id: VtkIdType,
        reference: i32,
        to_split: i32,
        point_id: &mut VtkIdType,
    ) {
        assert!(e1 != e2, "pre: not degenerated edge");
        order_edge(&mut e1, &mut e2);
        let pos = self.edge_hash(e1, e2);

        let mut entry = EdgeEntry {
            e1,
            e2,
            reference,
            to_split,
            cell_id,
            ..EdgeEntry::default()
        };
        if entry.to_split != 0 {
            entry.point_id = self.last_point_id;
            *point_id = self.last_point_id;
            self.last_point_id += 1;
        } else {
            entry.point_id = -1;
            *point_id = -1;
        }

        self.edge_table[pos].push(entry);
    }

    /// VTK: `vtkGenericEdgeTable::RemoveEdge`.
    pub fn remove_edge(&mut self, mut e1: VtkIdType, mut e2: VtkIdType) -> i32 {
        order_edge(&mut e1, &mut e2);
        let pos = self.edge_hash(e1, e2);
        let mut found = false;
        let mut reference = 0;
        let mut split_point_to_remove = None;

        self.edge_table[pos].retain_mut(|entry| {
            if entry.e1 == e1 && entry.e2 == e2 {
                entry.reference -= 1;
                found = true;
                reference = entry.reference;
                if entry.reference == 0 && entry.to_split != 0 {
                    assert!(entry.point_id >= 0, "check: positive id");
                    split_point_to_remove = Some(entry.point_id);
                }
            }
            !(entry.e1 == e1 && entry.e2 == e2 && entry.reference == 0)
        });

        assert!(found, "check: edge entry exists");
        if let Some(point_id) = split_point_to_remove {
            self.remove_point(point_id);
        }
        reference
    }

    /// VTK: `vtkGenericEdgeTable::CheckEdge`.
    pub fn check_edge(
        &self,
        mut e1: VtkIdType,
        mut e2: VtkIdType,
        point_id: &mut VtkIdType,
    ) -> i32 {
        order_edge(&mut e1, &mut e2);
        let pos = self.edge_hash(e1, e2);
        for entry in &self.edge_table[pos] {
            if entry.e1 == e1 && entry.e2 == e2 {
                *point_id = entry.point_id;
                return entry.to_split;
            }
        }
        -1
    }

    /// VTK: `vtkGenericEdgeTable::IncrementEdgeReferenceCount`.
    pub fn increment_edge_reference_count(
        &mut self,
        mut e1: VtkIdType,
        mut e2: VtkIdType,
        cell_id: VtkIdType,
    ) -> i32 {
        order_edge(&mut e1, &mut e2);
        let pos = self.edge_hash(e1, e2);
        for entry in &mut self.edge_table[pos] {
            if entry.e1 == e1 && entry.e2 == e2 {
                if entry.cell_id == cell_id {
                    entry.reference += 1;
                } else {
                    entry.cell_id = cell_id;
                }
                return -1;
            }
        }
        panic!("check: edge entry exists");
    }

    /// VTK: `vtkGenericEdgeTable::CheckEdgeReferenceCount`.
    pub fn check_edge_reference_count(&self, mut e1: VtkIdType, mut e2: VtkIdType) -> i32 {
        order_edge(&mut e1, &mut e2);
        let pos = self.edge_hash(e1, e2);
        for entry in &self.edge_table[pos] {
            if entry.e1 == e1 && entry.e2 == e2 {
                assert!(entry.reference >= 0, "check: positive reference");
                return entry.reference;
            }
        }
        panic!("check: edge entry exists");
    }

    /// VTK: `vtkGenericEdgeTable::Initialize`.
    pub fn initialize(&mut self, start: VtkIdType) {
        if self.last_point_id != 0 {
            return;
        }
        self.last_point_id = start;
    }

    /// VTK: `vtkGenericEdgeTable::GetNumberOfComponents`.
    pub fn get_number_of_components(&self) -> i32 {
        self.number_of_components
    }

    /// VTK: `vtkGenericEdgeTable::SetNumberOfComponents`.
    pub fn set_number_of_components(&mut self, count: i32) {
        assert!(count > 0, "pre: positive_count");
        self.number_of_components = count;
    }

    /// VTK: `vtkGenericEdgeTable::CheckPoint`.
    pub fn check_point(&self, point_id: VtkIdType) -> i32 {
        let pos = self.point_hash(point_id);
        for entry in &self.hash_points[pos] {
            if entry.point_id == point_id {
                return 1;
            }
        }
        0
    }

    /// VTK: `vtkGenericEdgeTable::CheckPoint(vtkIdType, double*, double*)`.
    pub fn check_point_values(
        &self,
        point_id: VtkIdType,
        point: &mut [f64; 3],
        scalar: &mut [f64],
    ) -> i32 {
        assert!(
            scalar.len() >= self.number_of_components as usize,
            "pre: scalar_size"
        );
        let pos = self.point_hash(point_id);
        for entry in &self.hash_points[pos] {
            if entry.point_id == point_id {
                *point = entry.coord;
                let count = self.number_of_components as usize;
                scalar[..count].copy_from_slice(&entry.scalar[..count]);
                return 1;
            }
        }
        0
    }

    /// VTK: `vtkGenericEdgeTable::InsertPoint`.
    pub fn insert_point(&mut self, point_id: VtkIdType, point: [f64; 3]) {
        let pos = self.point_hash(point_id);
        let mut entry = PointEntry::new(self.number_of_components);
        entry.point_id = point_id;
        entry.coord = point;
        entry.reference = 1;
        self.hash_points[pos].push(entry);
    }

    /// VTK: `vtkGenericEdgeTable::InsertPointAndScalar`.
    pub fn insert_point_and_scalar(
        &mut self,
        point_id: VtkIdType,
        point: [f64; 3],
        scalar: &[f64],
    ) {
        assert!(
            scalar.len() >= self.number_of_components as usize,
            "pre: scalar_size"
        );
        let pos = self.point_hash(point_id);
        let mut entry = PointEntry::new(self.number_of_components);
        entry.point_id = point_id;
        entry.coord = point;
        entry
            .scalar
            .copy_from_slice(&scalar[..self.number_of_components as usize]);
        entry.reference = 1;
        self.hash_points[pos].push(entry);
    }

    /// VTK: `vtkGenericEdgeTable::RemovePoint`.
    pub fn remove_point(&mut self, point_id: VtkIdType) {
        let pos = self.point_hash(point_id);
        let mut found = false;
        self.hash_points[pos].retain_mut(|entry| {
            if entry.point_id == point_id {
                entry.reference -= 1;
                found = true;
            }
            !(entry.point_id == point_id && entry.reference == 0)
        });
        let _ = found;
    }

    /// VTK: `vtkGenericEdgeTable::IncrementPointReferenceCount`.
    pub fn increment_point_reference_count(&mut self, point_id: VtkIdType) {
        let pos = self.point_hash(point_id);
        let mut found = false;
        for entry in &mut self.hash_points[pos] {
            if entry.point_id == point_id {
                entry.reference += 1;
                found = true;
            }
        }
        let _ = found;
    }

    /// VTK: `vtkGenericEdgeTable::DumpTable`.
    pub fn dump_table(&self) -> String {
        let mut out = self.dump_edges();
        out.push_str(&self.dump_points());
        out
    }

    /// VTK: `vtkGenericEdgeTable::LoadFactor`.
    pub fn load_factor(&self) -> String {
        format!("{}{}", self.edge_load_factor(), self.point_load_factor())
    }

    /// VTK: `vtkObjectBase::GetClassName`.
    pub fn get_class_name(&self) -> &'static str {
        self.object.get_class_name()
    }

    /// VTK: `vtkObject::Modified`.
    pub fn modified(&mut self) {
        self.object.modified();
    }

    /// VTK: `vtkObject::GetMTime`.
    pub fn get_m_time(&self) -> VtkMTimeType {
        self.object.get_m_time()
    }

    /// VTK: `vtkGenericEdgeTable::HashFunction(vtkIdType, vtkIdType)`.
    fn edge_hash(&self, e1: VtkIdType, e2: VtkIdType) -> usize {
        ((e1 + e2) % self.edge_modulo) as usize
    }

    /// VTK: `vtkGenericEdgeTable::HashFunction(vtkIdType)`.
    fn point_hash(&self, point_id: VtkIdType) -> usize {
        (point_id % self.point_modulo) as usize
    }

    /// VTK: `vtkEdgeTableEdge::DumpEdges`.
    fn dump_edges(&self) -> String {
        let mut out = String::new();
        for bucket in &self.edge_table {
            for entry in bucket {
                out.push_str(&format!(
                    "EdgeEntry: ({},{}) {},{},{}\n",
                    entry.e1, entry.e2, entry.reference, entry.to_split, entry.point_id
                ));
            }
        }
        out
    }

    /// VTK: `vtkEdgeTablePoints::DumpPoints`.
    fn dump_points(&self) -> String {
        let mut out = String::new();
        for bucket in &self.hash_points {
            for entry in bucket {
                out.push_str(&format!(
                    "PointEntry: {} {}:({},{},{})\n",
                    entry.point_id, entry.reference, entry.coord[0], entry.coord[1], entry.coord[2]
                ));
            }
        }
        out
    }

    /// VTK: `vtkEdgeTableEdge::LoadFactor`.
    fn edge_load_factor(&self) -> String {
        let entries: usize = self.edge_table.iter().map(Vec::len).sum();
        let bins = self
            .edge_table
            .iter()
            .filter(|bucket| !bucket.is_empty())
            .count();
        format!(
            "EdgeTableEdge:\n{},{},{},{}\n",
            self.edge_table.len(),
            entries,
            bins,
            self.edge_modulo
        )
    }

    /// VTK: `vtkEdgeTablePoints::LoadFactor`.
    fn point_load_factor(&self) -> String {
        let entries: usize = self.hash_points.iter().map(Vec::len).sum();
        let bins = self
            .hash_points
            .iter()
            .filter(|bucket| !bucket.is_empty())
            .count();
        let bucket_counts = self
            .hash_points
            .iter()
            .map(|bucket| bucket.len().to_string())
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "EdgeTablePoints:\n{}\n{},{},{},{}\n",
            bucket_counts,
            self.hash_points.len(),
            entries,
            bins,
            self.point_modulo
        )
    }
}

impl Default for GenericEdgeTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleCellTessellatorEdgeTableApi for GenericEdgeTable {
    fn initialize(&mut self, start: VtkIdType) {
        GenericEdgeTable::initialize(self, start);
    }

    fn set_number_of_components(&mut self, count: i32) {
        GenericEdgeTable::set_number_of_components(self, count);
    }

    fn check_point(&self, point_id: VtkIdType) -> bool {
        GenericEdgeTable::check_point(self, point_id) != 0
    }

    fn check_point_values(
        &self,
        point_id: VtkIdType,
        point: &mut [f64; 3],
        scalars: &mut [f64],
    ) -> bool {
        GenericEdgeTable::check_point_values(self, point_id, point, scalars) != 0
    }

    fn insert_point_and_scalar(&mut self, point_id: VtkIdType, point: [f64; 3], scalars: &[f64]) {
        GenericEdgeTable::insert_point_and_scalar(self, point_id, point, scalars);
    }

    fn increment_point_reference_count(&mut self, point_id: VtkIdType) {
        GenericEdgeTable::increment_point_reference_count(self, point_id);
    }

    fn remove_point(&mut self, point_id: VtkIdType) {
        GenericEdgeTable::remove_point(self, point_id);
    }

    fn check_edge(&self, left_id: VtkIdType, right_id: VtkIdType, point_id: &mut VtkIdType) -> i32 {
        GenericEdgeTable::check_edge(self, left_id, right_id, point_id)
    }

    fn insert_edge_with_point(
        &mut self,
        left_id: VtkIdType,
        right_id: VtkIdType,
        cell_id: VtkIdType,
        reference_count: i32,
        point_id: &mut VtkIdType,
    ) {
        GenericEdgeTable::insert_edge_with_point(
            self,
            left_id,
            right_id,
            cell_id,
            reference_count,
            point_id,
        );
    }

    fn insert_edge(
        &mut self,
        left_id: VtkIdType,
        right_id: VtkIdType,
        cell_id: VtkIdType,
        reference_count: i32,
    ) {
        GenericEdgeTable::insert_edge(self, left_id, right_id, cell_id, reference_count);
    }

    fn increment_edge_reference_count(
        &mut self,
        left_id: VtkIdType,
        right_id: VtkIdType,
        cell_id: VtkIdType,
    ) {
        GenericEdgeTable::increment_edge_reference_count(self, left_id, right_id, cell_id);
    }

    fn remove_edge(&mut self, left_id: VtkIdType, right_id: VtkIdType) {
        GenericEdgeTable::remove_edge(self, left_id, right_id);
    }
}

fn order_edge(e1: &mut VtkIdType, e2: &mut VtkIdType) {
    if *e1 > *e2 {
        std::mem::swap(e1, e2);
    }
}
