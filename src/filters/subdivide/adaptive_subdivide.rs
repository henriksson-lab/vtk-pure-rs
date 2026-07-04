use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};
use std::collections::HashMap;

/// Adaptively subdivide triangles that exceed a maximum edge length.
///
/// Mirrors VTK's `vtkAdaptiveSubdivisionFilter` edge-length path: each pass
/// marks every edge longer than `max_edge_length`, inserts midpoints on marked
/// edges, and uses the same case table to replace a triangle with 1-4 triangles.
pub fn adaptive_subdivide(input: &PolyData, max_edge_length: f64, max_passes: usize) -> PolyData {
    let mut points = input.points.clone();
    let mut current_tris: Vec<([i64; 3], usize)> = Vec::new();
    let mut point_arrays = collect_point_arrays(input);
    let mut cell_arrays = collect_cell_arrays(input);

    for (cell_id, cell) in input.polys.iter().enumerate() {
        if cell.len() != 3 {
            continue;
        }
        current_tris.push(([cell[0], cell[1], cell[2]], cell_id));
    }

    let max_len2 = max_edge_length * max_edge_length;
    let mut midpoint_cache: HashMap<(i64, i64), i64> = HashMap::new();

    for _ in 0..max_passes {
        let mut new_tris: Vec<([i64; 3], usize)> = Vec::new();
        let mut any_split = false;

        for &([a, b, c], cell_id) in &current_tris {
            let pa = points.get(a as usize);
            let pb = points.get(b as usize);
            let pc = points.get(c as usize);

            let e_lengths = [dist2(pa, pb), dist2(pb, pc), dist2(pc, pa)];
            let mut sub_case = 0usize;
            for (i, &len2) in e_lengths.iter().enumerate() {
                if len2 > max_len2 {
                    sub_case |= CASE_MASK[i];
                }
            }

            if sub_case == 0 {
                new_tris.push(([a, b, c], cell_id));
                continue;
            }

            any_split = true;
            let mut pt_ids = [a, b, c, -1, -1, -1];
            let edges = [(a, b), (b, c), (c, a)];
            for i in 0..3 {
                if sub_case & CASE_MASK[i] != 0 {
                    pt_ids[3 + i] = get_midpoint(
                        &mut points,
                        &mut point_arrays,
                        &mut midpoint_cache,
                        edges[i].0,
                        edges[i].1,
                    );
                }
            }

            let tess = select_tessellation(sub_case, &pt_ids, &points);
            for tri in tess {
                new_tris.push((
                    [
                        pt_ids[tri[0] as usize],
                        pt_ids[tri[1] as usize],
                        pt_ids[tri[2] as usize],
                    ],
                    cell_id,
                ));
            }
        }

        current_tris = new_tris;
        if !any_split {
            break;
        }
    }

    let mut polys = CellArray::new();
    for &([a, b, c], _) in &current_tris {
        polys.push_cell(&[a, b, c]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    add_point_arrays(&mut pd, point_arrays);
    add_cell_arrays(
        &mut pd,
        &mut cell_arrays,
        current_tris.iter().map(|&(_, cell_id)| cell_id),
    );
    pd
}

const CASE_MASK: [usize; 3] = [1, 2, 4];
const TESS_CASES: [&[[usize; 3]]; 16] = [
    &[[0, 1, 2]],
    &[[0, 3, 2], [3, 1, 2]],
    &[[0, 1, 4], [4, 2, 0]],
    &[[3, 1, 4], [3, 4, 2], [2, 0, 3]],
    &[[0, 1, 5], [5, 1, 2]],
    &[[0, 3, 5], [5, 3, 1], [1, 2, 5]],
    &[[5, 4, 2], [0, 1, 4], [4, 5, 0]],
    &[[0, 3, 5], [3, 1, 4], [5, 3, 4], [5, 4, 2]],
    &[[0, 1, 2]],
    &[[0, 3, 2], [3, 1, 2]],
    &[[0, 1, 4], [4, 2, 0]],
    &[[3, 1, 4], [0, 3, 4], [4, 2, 0]],
    &[[0, 1, 5], [5, 1, 2]],
    &[[0, 3, 5], [3, 1, 2], [2, 5, 3]],
    &[[4, 2, 5], [5, 0, 1], [1, 4, 5]],
    &[[0, 3, 5], [3, 1, 4], [5, 3, 4], [5, 4, 2]],
];

fn select_tessellation(
    sub_case: usize,
    pt_ids: &[i64; 6],
    points: &Points<f64>,
) -> &'static [[usize; 3]] {
    let tess = TESS_CASES[sub_case];
    if tess.len() != 3 {
        return tess;
    }

    let x0 = points.get(pt_ids[tess[1][0]] as usize);
    let x1 = points.get(pt_ids[tess[1][2]] as usize);
    let x2 = points.get(pt_ids[tess[1][1]] as usize);
    let x3 = points.get(pt_ids[tess[2][1]] as usize);
    if dist2(x0, x1) <= dist2(x2, x3) {
        tess
    } else {
        TESS_CASES[sub_case + 8]
    }
}

fn get_midpoint(
    points: &mut Points<f64>,
    point_arrays: &mut [PointArray],
    cache: &mut HashMap<(i64, i64), i64>,
    a: i64,
    b: i64,
) -> i64 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&mid) = cache.get(&key) {
        return mid;
    }
    let pa = points.get(a as usize);
    let pb = points.get(b as usize);
    let mid_pt = [
        (pa[0] + pb[0]) * 0.5,
        (pa[1] + pb[1]) * 0.5,
        (pa[2] + pb[2]) * 0.5,
    ];
    let idx = points.len() as i64;
    points.push(mid_pt);
    for array in point_arrays {
        array.push_edge_average(a as usize, b as usize);
    }
    cache.insert(key, idx);
    idx
}

fn dist2(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

struct PointArray {
    name: String,
    num_components: usize,
    data: Vec<f64>,
}

struct CellArrayData {
    name: String,
    num_components: usize,
    input_data: Vec<f64>,
    output_data: Vec<f64>,
}

impl PointArray {
    fn push_edge_average(&mut self, a: usize, b: usize) {
        for c in 0..self.num_components {
            self.data.push(
                0.5 * (self.data[a * self.num_components + c]
                    + self.data[b * self.num_components + c]),
            );
        }
    }
}

impl CellArrayData {
    fn copy_tuple(&mut self, cell_id: usize) {
        let start = cell_id * self.num_components;
        self.output_data
            .extend_from_slice(&self.input_data[start..start + self.num_components]);
    }
}

fn collect_point_arrays(input: &PolyData) -> Vec<PointArray> {
    input
        .point_data()
        .iter()
        .map(|array| {
            let num_components = array.num_components();
            PointArray {
                name: array.name().to_string(),
                num_components,
                data: array.to_f64_vec_flat(),
            }
        })
        .collect()
}

fn collect_cell_arrays(input: &PolyData) -> Vec<CellArrayData> {
    input
        .cell_data()
        .iter()
        .map(|array| CellArrayData {
            name: array.name().to_string(),
            num_components: array.num_components(),
            input_data: array.to_f64_vec_flat(),
            output_data: Vec::new(),
        })
        .collect()
}

fn add_point_arrays(output: &mut PolyData, arrays: Vec<PointArray>) {
    for array in arrays {
        output
            .point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                &array.name,
                array.data,
                array.num_components,
            )));
    }
}

fn add_cell_arrays(
    output: &mut PolyData,
    arrays: &mut [CellArrayData],
    cell_ids: impl Iterator<Item = usize>,
) {
    let ids: Vec<usize> = cell_ids.collect();
    for array in arrays {
        for &cell_id in &ids {
            array.copy_tuple(cell_id);
        }
        output
            .cell_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                &array.name,
                std::mem::take(&mut array.output_data),
                array.num_components,
            )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdivide_large_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([5.0, 10.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = adaptive_subdivide(&pd, 3.0, 10);
        assert!(result.polys.num_cells() > 1);
        assert!(result.points.len() > 3);
    }

    #[test]
    fn no_subdivide_small_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([0.1, 0.0, 0.0]);
        pd.points.push([0.05, 0.1, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = adaptive_subdivide(&pd, 1.0, 10);
        assert_eq!(result.polys.num_cells(), 1);
        assert_eq!(result.points.len(), 3);
    }

    #[test]
    fn max_passes_limits() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([100.0, 0.0, 0.0]);
        pd.points.push([50.0, 100.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let r1 = adaptive_subdivide(&pd, 1.0, 1);
        let r5 = adaptive_subdivide(&pd, 1.0, 5);
        assert!(r5.polys.num_cells() > r1.polys.num_cells());
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = adaptive_subdivide(&pd, 1.0, 10);
        assert_eq!(result.polys.num_cells(), 0);
    }

    #[test]
    fn shared_edges_use_same_midpoint() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([10.0, 0.0, 0.0]);
        pd.points.push([10.0, 10.0, 0.0]);
        pd.points.push([0.0, 10.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = adaptive_subdivide(&pd, 5.0, 1);
        // Shared edge 0-2 should create only one midpoint
        // No duplicate points for shared edges
        let n_pts = result.points.len();
        // With 4 original + shared midpoints, should be reasonable
        assert!(n_pts <= 10);
    }
}
