use crate::data::{CellArray, DataArray, Points, PolyData};
use std::collections::HashMap;

/// Extract iso-lines (contour lines) from a scalar field on a mesh surface.
///
/// For each triangle, finds edges where the scalar field crosses `iso_value`
/// and produces line segments connecting those crossing points.
/// Returns a PolyData containing line cells representing the iso-contour.
pub fn iso_lines(input: &PolyData, array_name: &str, iso_value: f64) -> PolyData {
    let n: usize = input.points.len();
    let scalars: Vec<f64> = match input.point_data().get_array(array_name) {
        Some(arr) => {
            if arr.num_tuples() < n {
                return PolyData::new();
            }
            let mut vals = vec![0.0f64; n];
            let mut buf = [0.0f64];
            for (i, val) in vals.iter_mut().enumerate() {
                arr.tuple_as_f64(i, &mut buf);
                *val = buf[0];
            }
            vals
        }
        None => return PolyData::new(),
    };

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut out_scalars = DataArray::<f64>::new("iso_value", 1);
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
        let nc: usize = cell.len();
        if nc < 3 {
            continue;
        }

        let base = cell[0] as usize;
        if base >= n {
            continue;
        }
        for tri in 1..nc - 1 {
            let ids = [base, cell[tri] as usize, cell[tri + 1] as usize];
            if ids.iter().any(|&id| id >= n) {
                continue;
            }
            let sv = [scalars[ids[0]], scalars[ids[1]], scalars[ids[2]]];
            let mut case_idx = 0usize;
            for i in 0..3 {
                if sv[i] >= iso_value {
                    case_idx |= 1 << i;
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
                        (iso_value - s0) / (s1 - s0)
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
                    line_ids[p] = push_iso_point(
                        &mut out_points,
                        &mut out_scalars,
                        &mut point_ids,
                        point,
                        iso_value,
                    ) as i64;
                }
                if line_ids[0] != line_ids[1] {
                    out_lines.push_cell(&line_ids);
                }
                edge += 2;
            }
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd.point_data_mut().add_array(out_scalars.into());
    pd
}

fn push_iso_point(
    out_points: &mut Points<f64>,
    out_scalars: &mut DataArray<f64>,
    point_ids: &mut HashMap<[u64; 3], usize>,
    point: [f64; 3],
    iso_value: f64,
) -> usize {
    let key = [point[0].to_bits(), point[1].to_bits(), point[2].to_bits()];
    if let Some(&idx) = point_ids.get(&key) {
        return idx;
    }
    let idx = out_points.len();
    out_points.push(point);
    out_scalars.push_tuple(&[iso_value]);
    point_ids.insert(key, idx);
    idx
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn iso_line_on_triangle() {
        // Triangle with scalar values 0, 0, 2 => iso at 1.0 crosses two edges
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = DataArray::from_vec("temp", vec![0.0, 0.0, 2.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = iso_lines(&pd, "temp", 1.0);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn iso_line_no_crossing() {
        // All scalar values above the iso-value
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = DataArray::from_vec("f", vec![5.0, 6.0, 7.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = iso_lines(&pd, "f", 1.0);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn iso_line_missing_array() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = iso_lines(&pd, "nonexistent", 0.5);
        assert_eq!(result.points.len(), 0);
        assert_eq!(result.lines.num_cells(), 0);
    }

    #[test]
    fn iso_line_through_exact_vertex() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = DataArray::from_vec("temp", vec![1.0, 0.0, 2.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = iso_lines(&pd, "temp", 1.0);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }

    #[test]
    fn iso_line_along_exact_edge() {
        let mut pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let scalars = DataArray::from_vec("temp", vec![1.0, 1.0, 0.0], 1);
        pd.point_data_mut().add_array(scalars.into());

        let result = iso_lines(&pd, "temp", 1.0);
        assert_eq!(result.lines.num_cells(), 1);
        assert_eq!(result.points.len(), 2);
    }
}
