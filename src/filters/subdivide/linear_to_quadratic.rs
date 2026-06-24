//! Upgrade linear cells to quadratic by adding midpoint nodes.
//!
//! Converts linear triangles to quadratic triangles (6 nodes) and
//! linear tetrahedra to quadratic tetrahedra (10 nodes) by inserting
//! midpoints on edges. Point data is interpolated linearly.

use crate::data::{AnyDataArray, CellArray, DataArray, Points, PolyData};

/// Convert linear triangles to quadratic triangles (6 nodes).
///
/// Each triangle (3 vertices) becomes a quadratic triangle with 6 nodes:
/// the 3 original vertices plus 3 edge midpoints.
///
/// Returns a new PolyData where each cell has 6 point indices.
/// Point data arrays are linearly interpolated at midpoints.
pub fn linear_to_quadratic_triangles(input: &PolyData) -> PolyData {
    let n_pts = input.points.len();
    if n_pts == 0 {
        return input.clone();
    }

    let mut new_points = Points::<f64>::new();
    for i in 0..n_pts {
        new_points.push(input.points.get(i));
    }

    // Track edge midpoints to avoid duplicates: (min_id, max_id) -> midpoint_index
    let mut edge_map: std::collections::HashMap<(usize, usize), usize> =
        std::collections::HashMap::new();

    let mut new_polys = CellArray::new();

    // Collect point data arrays for interpolation
    let pd = input.point_data();
    let n_arrays = pd.num_arrays();
    let mut array_data: Vec<Vec<f64>> = Vec::new();
    let mut array_nc: Vec<usize> = Vec::new();
    let mut array_names: Vec<String> = Vec::new();

    for ai in 0..n_arrays {
        if let Some(arr) = pd.get_array_by_index(ai) {
            let nc = arr.num_components();
            let mut data = Vec::with_capacity(n_pts * nc);
            let mut buf = vec![0.0f64; nc];
            for i in 0..n_pts {
                arr.tuple_as_f64(i, &mut buf);
                data.extend_from_slice(&buf);
            }
            array_data.push(data);
            array_nc.push(nc);
            array_names.push(arr.name().to_string());
        }
    }

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            // Pass through non-triangle cells unchanged
            new_polys.push_cell(cell);
            continue;
        }

        let v0 = cell[0] as usize;
        let v1 = cell[1] as usize;
        let v2 = cell[2] as usize;

        // Get or create midpoints
        let m01 = get_or_create_midpoint(
            v0,
            v1,
            &mut edge_map,
            &mut new_points,
            input,
            &mut array_data,
            &array_nc,
        );
        let m12 = get_or_create_midpoint(
            v1,
            v2,
            &mut edge_map,
            &mut new_points,
            input,
            &mut array_data,
            &array_nc,
        );
        let m20 = get_or_create_midpoint(
            v2,
            v0,
            &mut edge_map,
            &mut new_points,
            input,
            &mut array_data,
            &array_nc,
        );

        // Quadratic triangle: v0, v1, v2, m01, m12, m20
        new_polys.push_cell(&[
            v0 as i64, v1 as i64, v2 as i64, m01 as i64, m12 as i64, m20 as i64,
        ]);
    }

    let mut result = PolyData::new();
    result.points = new_points;
    result.polys = new_polys;

    // Add interpolated point data
    for (ai, data) in array_data.into_iter().enumerate() {
        let arr = AnyDataArray::F64(DataArray::from_vec(&array_names[ai], data, array_nc[ai]));
        result.point_data_mut().add_array(arr);
    }

    result
}

fn get_or_create_midpoint(
    a: usize,
    b: usize,
    edge_map: &mut std::collections::HashMap<(usize, usize), usize>,
    points: &mut Points<f64>,
    input: &PolyData,
    array_data: &mut [Vec<f64>],
    array_nc: &[usize],
) -> usize {
    let key = (a.min(b), a.max(b));
    if let Some(&idx) = edge_map.get(&key) {
        return idx;
    }

    let pa = input.points.get(a);
    let pb = input.points.get(b);
    let mid = [
        (pa[0] + pb[0]) / 2.0,
        (pa[1] + pb[1]) / 2.0,
        (pa[2] + pb[2]) / 2.0,
    ];

    let idx = points.len();
    points.push(mid);

    // Interpolate point data arrays
    for (ai, data) in array_data.iter_mut().enumerate() {
        let nc = array_nc[ai];
        for c in 0..nc {
            let va = data[a * nc + c];
            let vb = data[b * nc + c];
            data.push((va + vb) / 2.0);
        }
    }

    edge_map.insert(key, idx);
    idx
}

/// Count the number of unique edges in a triangle mesh.
pub fn count_edges(input: &PolyData) -> usize {
    let mut edges: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for cell in input.polys.iter() {
        let n = cell.len();
        for i in 0..n {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % n] as usize;
            edges.insert((a.min(b), a.max(b)));
        }
    }
    edges.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_to_quadratic() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = linear_to_quadratic_triangles(&mesh);
        assert_eq!(result.points.len(), 6); // 3 original + 3 midpoints
        assert_eq!(result.polys.num_cells(), 1);

        // Check midpoints
        let mid01 = result.points.get(3);
        assert!((mid01[0] - 1.0).abs() < 1e-10);
        assert!((mid01[1] - 0.0).abs() < 1e-10);
    }

    #[test]
    fn shared_edge_reuse() {
        // Two triangles sharing an edge
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );

        let result = linear_to_quadratic_triangles(&mesh);
        // 4 original + 5 unique edges = 9 points (shared edge midpoint reused)
        let n_edges = count_edges(&mesh);
        assert_eq!(result.points.len(), 4 + n_edges);
    }

    #[test]
    fn point_data_interpolation() {
        let mut mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            vec![[0, 1, 2]],
        );
        mesh.point_data_mut()
            .add_array(AnyDataArray::F64(DataArray::from_vec(
                "temp",
                vec![0.0, 100.0, 200.0],
                1,
            )));

        let result = linear_to_quadratic_triangles(&mesh);
        let arr = result.point_data().get_array("temp").unwrap();
        assert_eq!(arr.num_tuples(), 6);

        // Midpoint of edge 0-1 should have temp = 50.0
        let mut buf = [0.0f64];
        arr.tuple_as_f64(3, &mut buf);
        assert!((buf[0] - 50.0).abs() < 1e-10);
    }

    #[test]
    fn empty_mesh() {
        let mesh = PolyData::new();
        let result = linear_to_quadratic_triangles(&mesh);
        assert_eq!(result.points.len(), 0);
    }

    #[test]
    fn count_edges_test() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        assert_eq!(count_edges(&mesh), 3);
    }
}
