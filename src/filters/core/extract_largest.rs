use crate::data::{AnyDataArray, CellArray, DataArray, DataSetAttributes, Points, PolyData};
use std::collections::HashMap;

/// Extract the largest connected component from a PolyData.
///
/// Matches VTK connectivity's largest-region mode by selecting the region
/// with the largest number of cells.
pub fn extract_largest(input: &PolyData) -> PolyData {
    extract_regions_by_size(input, 1)
        .into_iter()
        .next()
        .unwrap_or_default()
}

/// Extract the N largest connected components.
pub fn extract_n_largest(input: &PolyData, n: usize) -> Vec<PolyData> {
    extract_regions_by_size(input, n)
}

fn extract_regions_by_size(input: &PolyData, n: usize) -> Vec<PolyData> {
    if n == 0 || input.points.is_empty() || input.total_cells() == 0 {
        return Vec::new();
    }

    let mut parent: Vec<usize> = (0..input.points.len()).collect();
    let mut rank = vec![0usize; input.points.len()];

    for cells in [&input.verts, &input.lines, &input.polys, &input.strips] {
        for cell in cells.iter() {
            if let Some((&first, rest)) = cell.split_first() {
                for &id in rest {
                    union(&mut parent, &mut rank, first as usize, id as usize);
                }
            }
        }
    }

    let mut regions: Vec<RegionCells> = Vec::new();
    let mut root_to_region: HashMap<usize, usize> = HashMap::new();
    collect_cells(
        &input.verts,
        CellKind::Verts,
        &mut parent,
        &mut root_to_region,
        &mut regions,
    );
    collect_cells(
        &input.lines,
        CellKind::Lines,
        &mut parent,
        &mut root_to_region,
        &mut regions,
    );
    collect_cells(
        &input.polys,
        CellKind::Polys,
        &mut parent,
        &mut root_to_region,
        &mut regions,
    );
    collect_cells(
        &input.strips,
        CellKind::Strips,
        &mut parent,
        &mut root_to_region,
        &mut regions,
    );

    regions.sort_by(|a, b| b.num_cells().cmp(&a.num_cells()));
    regions
        .into_iter()
        .take(n)
        .map(|region| build_region(input, &region))
        .collect()
}

#[derive(Clone, Copy)]
enum CellKind {
    Verts,
    Lines,
    Polys,
    Strips,
}

#[derive(Default)]
struct RegionCells {
    verts: Vec<usize>,
    lines: Vec<usize>,
    polys: Vec<usize>,
    strips: Vec<usize>,
}

impl RegionCells {
    fn push(&mut self, kind: CellKind, cell_id: usize) {
        match kind {
            CellKind::Verts => self.verts.push(cell_id),
            CellKind::Lines => self.lines.push(cell_id),
            CellKind::Polys => self.polys.push(cell_id),
            CellKind::Strips => self.strips.push(cell_id),
        }
    }

    fn num_cells(&self) -> usize {
        self.verts.len() + self.lines.len() + self.polys.len() + self.strips.len()
    }
}

fn collect_cells(
    cells: &CellArray,
    kind: CellKind,
    parent: &mut [usize],
    root_to_region: &mut HashMap<usize, usize>,
    regions: &mut Vec<RegionCells>,
) {
    for (cell_id, cell) in cells.iter().enumerate() {
        let Some(&first) = cell.first() else {
            continue;
        };
        let root = find(parent, first as usize);
        let region_id = *root_to_region.entry(root).or_insert_with(|| {
            regions.push(RegionCells::default());
            regions.len() - 1
        });
        regions[region_id].push(kind, cell_id);
    }
}

fn build_region(input: &PolyData, region: &RegionCells) -> PolyData {
    let mut point_map: HashMap<usize, i64> = HashMap::new();
    let mut point_ids = Vec::new();
    let mut points = Points::<f64>::new();
    let verts = remap_cells(
        &input.verts,
        &region.verts,
        input,
        &mut point_map,
        &mut point_ids,
        &mut points,
    );
    let lines = remap_cells(
        &input.lines,
        &region.lines,
        input,
        &mut point_map,
        &mut point_ids,
        &mut points,
    );
    let polys = remap_cells(
        &input.polys,
        &region.polys,
        input,
        &mut point_map,
        &mut point_ids,
        &mut points,
    );
    let strips = remap_cells(
        &input.strips,
        &region.strips,
        input,
        &mut point_map,
        &mut point_ids,
        &mut points,
    );

    let mut output = PolyData::new();
    output.points = points;
    output.verts = verts;
    output.lines = lines;
    output.polys = polys;
    output.strips = strips;

    copy_point_data(input, &mut output, &point_ids);
    copy_cell_data(input, &mut output, region);
    for array in input.field_data().iter() {
        output.field_data_mut().add_array(array.clone());
    }
    output
}

fn remap_cells(
    cells: &CellArray,
    cell_ids: &[usize],
    input: &PolyData,
    point_map: &mut HashMap<usize, i64>,
    point_ids: &mut Vec<usize>,
    points: &mut Points<f64>,
) -> CellArray {
    let mut output = CellArray::new();
    for &cell_id in cell_ids {
        let mapped: Vec<i64> = cells
            .cell(cell_id)
            .iter()
            .map(|&id| {
                let old_id = id as usize;
                *point_map.entry(old_id).or_insert_with(|| {
                    let new_id = points.len() as i64;
                    points.push(input.points.get(old_id));
                    point_ids.push(old_id);
                    new_id
                })
            })
            .collect();
        output.push_cell(&mapped);
    }
    output
}

fn copy_point_data(input: &PolyData, output: &mut PolyData, point_ids: &[usize]) {
    copy_tuple_subset(
        input.point_data(),
        output.point_data_mut(),
        point_ids,
        input.points.len(),
    );
}

fn copy_cell_data(input: &PolyData, output: &mut PolyData, region: &RegionCells) {
    let verts_offset = 0;
    let lines_offset = input.verts.num_cells();
    let polys_offset = lines_offset + input.lines.num_cells();
    let strips_offset = polys_offset + input.polys.num_cells();
    let mut cell_ids = Vec::with_capacity(region.num_cells());
    cell_ids.extend(region.verts.iter().map(|&id| verts_offset + id));
    cell_ids.extend(region.lines.iter().map(|&id| lines_offset + id));
    cell_ids.extend(region.polys.iter().map(|&id| polys_offset + id));
    cell_ids.extend(region.strips.iter().map(|&id| strips_offset + id));
    copy_tuple_subset(
        input.cell_data(),
        output.cell_data_mut(),
        &cell_ids,
        input.total_cells(),
    );
}

fn copy_tuple_subset(
    source: &DataSetAttributes,
    target: &mut DataSetAttributes,
    tuple_ids: &[usize],
    expected_tuples: usize,
) {
    for array in source.iter() {
        if array.num_tuples() != expected_tuples {
            continue;
        }
        if let Some(subset) = subset_array(array, tuple_ids) {
            let name = subset.name().to_string();
            target.add_array(subset);
            copy_active_attribute(source, target, &name);
        }
    }
}

fn subset_array(array: &AnyDataArray, tuple_ids: &[usize]) -> Option<AnyDataArray> {
    macro_rules! subset_variant {
        ($variant:ident) => {{
            let AnyDataArray::$variant(a) = array else {
                unreachable!();
            };
            let nc = a.num_components();
            let mut data = Vec::with_capacity(tuple_ids.len() * nc);
            for &tuple_id in tuple_ids {
                if tuple_id >= a.num_tuples() {
                    return None;
                }
                data.extend_from_slice(a.tuple(tuple_id));
            }
            Some(AnyDataArray::$variant(DataArray::from_vec(
                a.name(),
                data,
                nc,
            )))
        }};
    }
    match array {
        AnyDataArray::F32(_) => subset_variant!(F32),
        AnyDataArray::F64(_) => subset_variant!(F64),
        AnyDataArray::I8(_) => subset_variant!(I8),
        AnyDataArray::I16(_) => subset_variant!(I16),
        AnyDataArray::I32(_) => subset_variant!(I32),
        AnyDataArray::I64(_) => subset_variant!(I64),
        AnyDataArray::U8(_) => subset_variant!(U8),
        AnyDataArray::U16(_) => subset_variant!(U16),
        AnyDataArray::U32(_) => subset_variant!(U32),
        AnyDataArray::U64(_) => subset_variant!(U64),
    }
}

fn copy_active_attribute(source: &DataSetAttributes, target: &mut DataSetAttributes, name: &str) {
    if source.scalars().map(|a| a.name()) == Some(name) {
        target.set_active_scalars(name);
    }
    if source.vectors().map(|a| a.name()) == Some(name) {
        target.set_active_vectors(name);
    }
    if source.normals().map(|a| a.name()) == Some(name) {
        target.set_active_normals(name);
    }
    if source.tcoords().map(|a| a.name()) == Some(name) {
        target.set_active_tcoords(name);
    }
    if source.tensors().map(|a| a.name()) == Some(name) {
        target.set_active_tensors(name);
    }
    if source.global_ids().map(|a| a.name()) == Some(name) {
        target.set_active_global_ids(name);
    }
    if source.pedigree_ids().map(|a| a.name()) == Some(name) {
        target.set_active_pedigree_ids(name);
    }
    if source.edge_flags().map(|a| a.name()) == Some(name) {
        target.set_active_edge_flags(name);
    }
    if source.tangents().map(|a| a.name()) == Some(name) {
        target.set_active_tangents(name);
    }
    if source.rational_weights().map(|a| a.name()) == Some(name) {
        target.set_active_rational_weights(name);
    }
    if source.higher_order_degrees().map(|a| a.name()) == Some(name) {
        target.set_active_higher_order_degrees(name);
    }
    if source.process_ids().map(|a| a.name()) == Some(name) {
        target.set_active_process_ids(name);
    }
}

fn find(parent: &mut [usize], mut x: usize) -> usize {
    while parent[x] != x {
        parent[x] = parent[parent[x]];
        x = parent[x];
    }
    x
}

fn union(parent: &mut [usize], rank: &mut [usize], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra == rb {
        return;
    }
    if rank[ra] < rank[rb] {
        parent[ra] = rb;
    } else if rank[ra] > rank[rb] {
        parent[rb] = ra;
    } else {
        parent[rb] = ra;
        rank[ra] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn largest_of_two() {
        let mut pd = PolyData::new();
        // Small component: 1 triangle
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.0, 0.1, 0.0]);
        // Large component: 2 triangles (more points)
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        pd.points.push([6.0, 1.0, 0.0]);
        pd.points.push([5.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.polys.push_cell(&[3, 5, 6]);

        let result = extract_largest(&pd);
        assert_eq!(result.polys.num_cells(), 2); // the larger component
    }

    #[test]
    fn extract_n() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.points.push([5.0, 0.0, 0.0]);
        pd.points.push([6.0, 0.0, 0.0]);
        pd.points.push([5.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);

        let results = extract_n_largest(&pd, 5); // ask for 5, only 2 exist
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = extract_largest(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn largest_uses_cell_count_not_point_count() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([3.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.5, 1.0, 0.0]);
        pd.points.push([10.5, -1.0, 0.0]);
        pd.lines.push_cell(&[0, 1, 2, 3]);
        pd.polys.push_cell(&[4, 5, 6]);
        pd.polys.push_cell(&[4, 7, 5]);

        let result = extract_largest(&pd);
        assert_eq!(result.polys.num_cells(), 2);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn largest_copies_point_and_cell_data() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([11.0, 0.0, 0.0]);
        pd.points.push([10.0, 1.0, 0.0]);
        pd.points.push([11.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[3, 4, 5]);
        pd.polys.push_cell(&[4, 6, 5]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "pid",
                vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                1,
            )));
        pd.point_data_mut().set_active_scalars("pid");
        pd.cell_data_mut()
            .add_array(AnyDataArray::I32(DataArray::from_vec(
                "cid",
                vec![10, 20, 30],
                1,
            )));

        let result = extract_largest(&pd);
        let point_ids = result.point_data().scalars().unwrap().to_f64_vec();
        let cell_ids = result.cell_data().get_array("cid").unwrap().to_f64_vec();
        assert_eq!(point_ids, vec![3.0, 4.0, 5.0, 6.0]);
        assert_eq!(cell_ids, vec![20.0, 30.0]);
    }
}
