use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Extract a level set (isocontour) of a scalar field on a triangle mesh.
///
/// Finds edges where the scalar crosses the isovalue and interpolates
/// the crossing point. Returns a PolyData with line segments.
pub fn mesh_level_set(input: &PolyData, array_name: &str, isovalue: f64) -> PolyData {
    let arr = match input.point_data().get_array(array_name) {
        Some(a) => a,
        None => return PolyData::new(),
    };

    let mut buf = [0.0f64];
    let n = input.points.len();
    if arr.num_tuples() < n {
        return PolyData::new();
    }
    let values: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            buf[0]
        })
        .collect();

    let mut out_pts = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut point_ids = HashMap::new();

    const LINE_CASES: [[i32; 3]; 8] = [
        [-1, -1, -1],
        [0, 2, -1],
        [1, 0, -1],
        [1, 2, -1],
        [2, 1, -1],
        [0, 1, -1],
        [2, 0, -1],
        [-1, -1, -1],
    ];
    const EDGES: [[usize; 2]; 3] = [[0, 1], [1, 2], [2, 0]];

    for cell in input.polys.iter() {
        if cell.len() < 3 {
            continue;
        }
        let base = cell[0] as usize;
        if base >= n {
            continue;
        }
        for tri in 1..cell.len() - 1 {
            let ids = [base, cell[tri] as usize, cell[tri + 1] as usize];
            if ids.iter().any(|&id| id >= n) {
                continue;
            }
            let sv = [values[ids[0]], values[ids[1]], values[ids[2]]];
            let mut case_idx = 0usize;
            for k in 0..3 {
                if sv[k] >= isovalue {
                    case_idx |= 1 << k;
                }
            }

            let mut edge = 0usize;
            while LINE_CASES[case_idx][edge] >= 0 {
                let mut line_ids = [0i64; 2];
                for p in 0..2 {
                    let edge_id = LINE_CASES[case_idx][edge + p] as usize;
                    let [e0, e1] = EDGES[edge_id];
                    let s0 = sv[e0];
                    let s1 = sv[e1];
                    let t = if (s1 - s0).abs() > 0.0 {
                        (isovalue - s0) / (s1 - s0)
                    } else {
                        0.0
                    };
                    let p0 = input.points.get(ids[e0]);
                    let p1 = input.points.get(ids[e1]);
                    let point = [
                        p0[0] + t * (p1[0] - p0[0]),
                        p0[1] + t * (p1[1] - p0[1]),
                        p0[2] + t * (p1[2] - p0[2]),
                    ];
                    line_ids[p] = insert_unique_point(&mut out_pts, &mut point_ids, point);
                }
                if line_ids[0] != line_ids[1] {
                    out_lines.push_cell(&line_ids);
                }
                edge += 2;
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_pts;
    pd.lines = out_lines;
    pd
}

fn insert_unique_point(
    points: &mut Points<f64>,
    ids: &mut HashMap<[u64; 3], i64>,
    point: [f64; 3],
) -> i64 {
    let key = [point[0].to_bits(), point[1].to_bits(), point[2].to_bits()];
    if let Some(&id) = ids.get(&key) {
        return id;
    }
    let id = points.len() as i64;
    points.push(point);
    ids.insert(key, id);
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{AnyDataArray, DataArray};

    #[test]
    fn contour_on_gradient() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.points.push([0.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![0.0, 2.0, 1.0, 0.0],
                1,
            )));

        let result = mesh_level_set(&pd, "f", 0.5);
        assert!(result.lines.num_cells() > 0);
    }

    #[test]
    fn no_crossing() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "f",
                vec![5.0, 5.0, 5.0],
                1,
            )));

        let result = mesh_level_set(&pd, "f", 0.0);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn missing_array() {
        let pd = PolyData::new();
        let result = mesh_level_set(&pd, "nope", 0.0);
        assert_eq!(result.lines.num_cells(), 0);
    }
}
