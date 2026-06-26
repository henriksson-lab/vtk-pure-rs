use crate::data::{AnyDataArray, DataArray, DataSetAttributes, Points, PolyData};
use crate::types::Scalar;

/// Extract connected components from a PolyData mesh.
///
/// Returns a Vec of PolyData, one per connected component, sorted by size (largest first).
pub fn extract_components(input: &PolyData) -> Vec<PolyData> {
    let n = input.points.len();
    if n == 0 {
        return Vec::new();
    }
    let cells = collect_cells(input);
    if cells.is_empty() {
        return Vec::new();
    }

    // Union-find
    let mut parent: Vec<usize> = (0..n).collect();
    let mut rank = vec![0usize; n];

    for cell in &cells {
        if cell.points.len() < 2 {
            continue;
        }
        let first = cell.points[0] as usize;
        for &pt in &cell.points[1..] {
            union(&mut parent, &mut rank, first, pt as usize);
        }
    }

    let mut component_cells: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();

    for (ci, cell) in cells.iter().enumerate() {
        if cell.points.is_empty() {
            continue;
        }
        let root = find(&mut parent, cell.points[0] as usize);
        component_cells.entry(root).or_default().push(ci);
    }

    // Sort by size (largest first)
    let mut components: Vec<Vec<usize>> = component_cells.into_values().collect();
    components.sort_by(|a, b| b.len().cmp(&a.len()));

    components
        .into_iter()
        .map(|cell_indices| build_component(input, &cells, n, &cell_indices))
        .collect()
}

/// Extract only the largest connected component.
pub fn extract_largest_component(input: &PolyData) -> PolyData {
    let mut components = extract_components(input);
    if components.is_empty() {
        PolyData::new()
    } else {
        components.remove(0)
    }
}

#[derive(Clone, Copy)]
enum CellKind {
    Vert,
    Line,
    Poly,
    Strip,
}

struct CellRef<'a> {
    kind: CellKind,
    global_id: usize,
    points: &'a [i64],
}

fn collect_cells(input: &PolyData) -> Vec<CellRef<'_>> {
    let mut cells = Vec::new();
    let mut global_id = 0usize;
    for (kind, array) in [
        (CellKind::Vert, &input.verts),
        (CellKind::Line, &input.lines),
        (CellKind::Poly, &input.polys),
        (CellKind::Strip, &input.strips),
    ] {
        for cell in array.iter() {
            cells.push(CellRef {
                kind,
                global_id,
                points: cell,
            });
            global_id += 1;
        }
    }
    cells
}

fn build_component(
    input: &PolyData,
    cells: &[CellRef<'_>],
    n_pts: usize,
    cell_indices: &[usize],
) -> PolyData {
    let mut pt_map: Vec<i64> = vec![-1; n_pts];
    let mut point_ids = Vec::new();
    let mut pts_flat: Vec<f64> = Vec::new();
    let mut cell_ids = Vec::new();
    let mut output = PolyData::new();

    for &ci in cell_indices {
        let cell = &cells[ci];
        let mut remapped = Vec::with_capacity(cell.points.len());
        for &old in cell.points {
            let old_id = old as usize;
            if pt_map[old_id] < 0 {
                pt_map[old_id] = (pts_flat.len() / 3) as i64;
                point_ids.push(old_id);
                let b = old_id * 3;
                pts_flat.extend_from_slice(&input.points.as_flat_slice()[b..b + 3]);
            }
            remapped.push(pt_map[old_id]);
        }

        match cell.kind {
            CellKind::Vert => output.verts.push_cell(&remapped),
            CellKind::Line => output.lines.push_cell(&remapped),
            CellKind::Poly => output.polys.push_cell(&remapped),
            CellKind::Strip => output.strips.push_cell(&remapped),
        }
        cell_ids.push(cell.global_id);
    }
    output.points = Points::from_flat_vec(pts_flat);
    copy_arrays_by_indices(input.point_data(), output.point_data_mut(), &point_ids);
    copy_arrays_by_indices(input.cell_data(), output.cell_data_mut(), &cell_ids);
    output
}

fn find(parent: &mut [usize], x: usize) -> usize {
    if parent[x] != x {
        parent[x] = find(parent, parent[x]);
    }
    parent[x]
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

fn copy_arrays_by_indices(
    input: &DataSetAttributes,
    output: &mut DataSetAttributes,
    indices: &[usize],
) {
    for arr in input.iter() {
        output.add_array(copy_array_by_indices(arr, indices));
    }
}

fn copy_array_by_indices(arr: &AnyDataArray, indices: &[usize]) -> AnyDataArray {
    macro_rules! copy {
        ($array:expr, $variant:ident) => {{
            AnyDataArray::$variant(copy_typed_array($array, indices))
        }};
    }
    match arr {
        AnyDataArray::F32(a) => copy!(a, F32),
        AnyDataArray::F64(a) => copy!(a, F64),
        AnyDataArray::I8(a) => copy!(a, I8),
        AnyDataArray::I16(a) => copy!(a, I16),
        AnyDataArray::I32(a) => copy!(a, I32),
        AnyDataArray::I64(a) => copy!(a, I64),
        AnyDataArray::U8(a) => copy!(a, U8),
        AnyDataArray::U16(a) => copy!(a, U16),
        AnyDataArray::U32(a) => copy!(a, U32),
        AnyDataArray::U64(a) => copy!(a, U64),
    }
}

fn copy_typed_array<T: Scalar>(array: &DataArray<T>, indices: &[usize]) -> DataArray<T> {
    let nc = array.num_components();
    let mut data = Vec::with_capacity(indices.len() * nc);
    for &idx in indices {
        data.extend_from_slice(array.tuple(idx));
    }
    DataArray::from_vec(array.name(), data, nc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_disconnected_triangles() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                // Separate triangle
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );

        let components = extract_components(&pd);
        assert_eq!(components.len(), 2);
        assert_eq!(components[0].polys.num_cells(), 1);
        assert_eq!(components[1].polys.num_cells(), 1);
    }

    #[test]
    fn single_connected_mesh() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );

        let components = extract_components(&pd);
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].polys.num_cells(), 2);
    }

    #[test]
    fn largest_component() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
                // Separate single triangle
                [10.0, 0.0, 0.0],
                [11.0, 0.0, 0.0],
                [10.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2], [4, 5, 6]],
        );

        let largest = extract_largest_component(&pd);
        assert_eq!(largest.polys.num_cells(), 2);
    }
}
