use crate::data::{AnyDataArray, DataArray, PolyData};

/// Compute ordered ring of vertices around each vertex.
///
/// For each vertex, finds the ordered sequence of adjacent vertices
/// following the face connectivity. Adds "RingSize" scalar (number of
/// ring neighbors). Useful for detecting irregular vertices.
pub fn compute_vertex_rings(input: &PolyData) -> PolyData {
    let n = input.points.len();
    if n == 0 {
        return input.clone();
    }

    let mut ring_sizes = vec![0.0f64; n];
    let cells: Vec<Vec<i64>> = input.polys.iter().map(|c| c.to_vec()).collect();

    for i in 0..n {
        let vid = i as i64;
        // Find faces containing this vertex
        let mut ring_edges: Vec<(i64, i64)> = Vec::new();
        for c in &cells {
            for j in 0..c.len() {
                if c[j] == vid {
                    let prev = c[(j + c.len() - 1) % c.len()];
                    let next = c[(j + 1) % c.len()];
                    ring_edges.push((prev, next));
                }
            }
        }

        // Count unique neighbors
        let mut neighbors = std::collections::HashSet::new();
        for &(a, b) in &ring_edges {
            neighbors.insert(a);
            neighbors.insert(b);
        }
        ring_sizes[i] = neighbors.len() as f64;
    }

    let mut pd = input.clone();
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "RingSize", ring_sizes, 1,
        )));
    pd
}

/// Detect irregular vertices (valence != 6 for triangle meshes).
/// Adds "IsIrregular" scalar (1=irregular, 0=regular).
pub fn detect_irregular_vertices(input: &PolyData, expected_valence: usize) -> PolyData {
    let result = compute_vertex_rings(input);
    let arr = match result.point_data().get_array("RingSize") {
        Some(a) => a,
        None => return input.clone(),
    };

    let n = arr.num_tuples();
    let mut buf = [0.0f64];
    let irregular: Vec<f64> = (0..n)
        .map(|i| {
            arr.tuple_as_f64(i, &mut buf);
            if (buf[0] as usize) != expected_valence && buf[0] > 0.0 {
                1.0
            } else {
                0.0
            }
        })
        .collect();

    let mut pd = result;
    pd.point_data_mut()
        .add_array(AnyDataArray::F64(DataArray::from_vec(
            "IsIrregular",
            irregular,
            1,
        )));
    pd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fan_ring_size() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]); // center
        for i in 0..6 {
            let a = std::f64::consts::PI * 2.0 * i as f64 / 6.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..6 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 6 + 1) as i64]);
        }

        let result = compute_vertex_rings(&pd);
        let arr = result.point_data().get_array("RingSize").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 6.0); // center has 6 ring neighbors
    }

    #[test]
    fn irregular_detection() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        for i in 0..5 {
            // 5-valent (irregular for triangle mesh)
            let a = std::f64::consts::PI * 2.0 * i as f64 / 5.0;
            pd.points.push([a.cos(), a.sin(), 0.0]);
        }
        for i in 0..5 {
            pd.polys
                .push_cell(&[0, (i + 1) as i64, ((i + 1) % 5 + 1) as i64]);
        }

        let result = detect_irregular_vertices(&pd, 6);
        let arr = result.point_data().get_array("IsIrregular").unwrap();
        let mut buf = [0.0f64];
        arr.tuple_as_f64(0, &mut buf);
        assert_eq!(buf[0], 1.0); // 5-valent is irregular
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = compute_vertex_rings(&pd);
        assert_eq!(result.points.len(), 0);
    }
}
