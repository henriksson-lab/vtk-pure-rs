use std::collections::HashMap;

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Assign colors to mesh edges based on their dihedral angle.
///
/// Each edge is classified as either "flat" (dihedral angle below the
/// threshold) or "sharp" (at or above the threshold).  Boundary edges
/// (used by only one face) are always classified as sharp.
///
/// Returns a new line-based PolyData with an "EdgeColor" 3-component RGB
/// cell data array.  Flat edges are colored green `[0, 1, 0]` and sharp
/// edges are colored red `[1, 0, 0]`.
pub fn color_edges_by_angle(input: &PolyData, sharp_threshold_deg: f64) -> PolyData {
    let cos_threshold: f64 = sharp_threshold_deg.to_radians().cos();

    // Compute face normals.
    let face_normals: Vec<[f64; 3]> = input
        .polys
        .iter()
        .map(|c| polygon_normal(&input.points, &c))
        .collect();

    // Build edge -> face mapping.
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (fi, cell) in input.polys.iter().enumerate() {
        let n: usize = cell.len();
        for i in 0..n {
            let a: i64 = cell[i];
            let b: i64 = cell[(i + 1) % n];
            let key: (i64, i64) = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut out_points = Points::<f64>::new();
    let mut out_lines = CellArray::new();
    let mut colors: Vec<f64> = Vec::new();
    let mut point_map: HashMap<i64, i64> = HashMap::new();

    for (&(a, b), faces) in &edge_faces {
        let is_sharp: bool = if faces.len() == 1 {
            true // boundary edge
        } else if faces.len() == 2 {
            let n1: [f64; 3] = face_normals[faces[0]];
            let n2: [f64; 3] = face_normals[faces[1]];
            let dot: f64 = n1[0] * n2[0] + n1[1] * n2[1] + n1[2] * n2[2];
            dot < cos_threshold
        } else {
            true // non-manifold
        };

        let ma: i64 = map_point(a, &input.points, &mut out_points, &mut point_map);
        let mb: i64 = map_point(b, &input.points, &mut out_points, &mut point_map);
        out_lines.push_cell(&[ma, mb]);

        if is_sharp {
            colors.extend_from_slice(&[1.0, 0.0, 0.0]); // red
        } else {
            colors.extend_from_slice(&[0.0, 1.0, 0.0]); // green
        }
    }

    let mut pd = PolyData::new();
    pd.points = out_points;
    pd.lines = out_lines;
    pd.cell_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "EdgeColor",
            colors,
            3,
        )));
    pd
}

fn map_point(
    id: i64,
    src: &Points<f64>,
    dst: &mut Points<f64>,
    map: &mut HashMap<i64, i64>,
) -> i64 {
    *map.entry(id).or_insert_with(|| {
        let idx: i64 = dst.len() as i64;
        dst.push(src.get(id as usize));
        idx
    })
}

fn polygon_normal(points: &Points<f64>, cell: &[i64]) -> [f64; 3] {
    let mut nx: f64 = 0.0;
    let mut ny: f64 = 0.0;
    let mut nz: f64 = 0.0;
    let n: usize = cell.len();
    if n < 3 {
        return [0.0, 0.0, 1.0];
    }
    for i in 0..n {
        let p: [f64; 3] = points.get(cell[i] as usize);
        let q: [f64; 3] = points.get(cell[(i + 1) % n] as usize);
        nx += (p[1] - q[1]) * (p[2] + q[2]);
        ny += (p[2] - q[2]) * (p[0] + q[0]);
        nz += (p[0] - q[0]) * (p[1] + q[1]);
    }
    let len: f64 = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-20 {
        [nx / len, ny / len, nz / len]
    } else {
        [0.0, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_edges_are_green() {
        // Two coplanar triangles sharing edge 1-2 with consistent winding.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 0, 3]],
        );
        let result = color_edges_by_angle(&pd, 30.0);
        let arr = result.cell_data().get_array("EdgeColor").unwrap();
        // The shared edge should be green (flat). Find it.
        let num: usize = arr.num_tuples();
        let mut found_green: bool = false;
        for i in 0..num {
            let mut c = [0.0f64; 3];
            arr.tuple_as_f64(i, &mut c);
            if c[1] > 0.9 && c[0] < 0.1 {
                found_green = true;
            }
        }
        assert!(found_green, "Expected at least one green (flat) edge");
    }

    #[test]
    fn sharp_edges_are_red() {
        // Two triangles at ~90 degrees.
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.0, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3]],
        );
        let result = color_edges_by_angle(&pd, 30.0);
        let arr = result.cell_data().get_array("EdgeColor").unwrap();
        // The shared edge should be red (sharp).
        let num: usize = arr.num_tuples();
        let mut found_red: bool = false;
        for i in 0..num {
            let mut c = [0.0f64; 3];
            arr.tuple_as_f64(i, &mut c);
            if c[0] > 0.9 && c[1] < 0.1 {
                found_red = true;
            }
        }
        assert!(found_red, "Expected at least one red (sharp) edge");
    }

    #[test]
    fn single_triangle_all_boundary() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = color_edges_by_angle(&pd, 30.0);
        let arr = result.cell_data().get_array("EdgeColor").unwrap();
        assert_eq!(arr.num_tuples(), 3); // 3 boundary edges
                                         // All boundary edges are sharp (red).
        for i in 0..3 {
            let mut c = [0.0f64; 3];
            arr.tuple_as_f64(i, &mut c);
            assert!(c[0] > 0.9, "boundary edge should be red");
        }
    }
}
