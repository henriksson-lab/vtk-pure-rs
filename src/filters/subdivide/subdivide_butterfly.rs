use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Modified butterfly subdivision.
///
/// Each triangle is split into 4 by inserting edge midpoints, similar to
/// Loop subdivision but using the butterfly stencil for smoother interpolation
/// on irregular meshes. Falls back to midpoint for boundary edges.
pub fn subdivide_butterfly(input: &PolyData) -> PolyData {
    let _n = input.points.len();
    let mut points = input.points.clone();
    let mut new_tris: Vec<[i64; 3]> = Vec::new();

    // Build edge-to-face adjacency
    let mut edge_faces: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    let tris: Vec<[i64; 3]> = input
        .polys
        .iter()
        .filter_map(|cell| {
            if cell.len() >= 3 {
                Some([cell[0], cell[1], cell[2]])
            } else {
                None
            }
        })
        .collect();

    for (fi, tri) in tris.iter().enumerate() {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_faces.entry(key).or_default().push(fi);
        }
    }

    let mut midpoint_cache: HashMap<(i64, i64), i64> = HashMap::new();

    for tri in &tris {
        let a = tri[0];
        let b = tri[1];
        let c = tri[2];

        let ab = get_butterfly_midpoint(&mut points, &mut midpoint_cache, &edge_faces, &tris, a, b);
        let bc = get_butterfly_midpoint(&mut points, &mut midpoint_cache, &edge_faces, &tris, b, c);
        let ca = get_butterfly_midpoint(&mut points, &mut midpoint_cache, &edge_faces, &tris, c, a);

        new_tris.push([a, ab, ca]);
        new_tris.push([b, bc, ab]);
        new_tris.push([c, ca, bc]);
        new_tris.push([ab, bc, ca]);
    }

    let mut polys = CellArray::new();
    for tri in &new_tris {
        polys.push_cell(&[tri[0], tri[1], tri[2]]);
    }

    let mut pd = PolyData::new();
    pd.points = points;
    pd.polys = polys;
    pd
}

fn get_butterfly_midpoint(
    points: &mut Points<f64>,
    cache: &mut HashMap<(i64, i64), i64>,
    edge_faces: &HashMap<(i64, i64), Vec<usize>>,
    tris: &[[i64; 3]],
    a: i64,
    b: i64,
) -> i64 {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&mid) = cache.get(&key) {
        return mid;
    }

    let pa = points.get(a as usize);
    let pb = points.get(b as usize);

    // Find opposite vertices
    let faces = edge_faces.get(&key);
    let is_boundary = faces.map(|f| f.len()).unwrap_or(0) < 2;

    let mid_pt = if is_boundary {
        // Simple midpoint for boundary edges
        [
            (pa[0] + pb[0]) * 0.5,
            (pa[1] + pb[1]) * 0.5,
            (pa[2] + pb[2]) * 0.5,
        ]
    } else {
        // Butterfly: midpoint + correction from opposite vertices
        // Standard butterfly weight: w = 1/2 for edge vertices, w = 1/8 for wing vertices
        let faces = faces.unwrap();
        let mut opp = Vec::new();
        for &fi in faces {
            let tri = &tris[fi];
            for &v in tri {
                if v != a && v != b {
                    opp.push(v);
                }
            }
        }

        if opp.len() >= 2 {
            let pc = points.get(opp[0] as usize);
            let pd_pt = points.get(opp[1] as usize);
            // Butterfly: 1/2*(a+b) + 1/8*(c+d)... simplified
            // Modified butterfly: (8a + 8b + 2c + 2d) / 20
            // But standard is: midpoint + 1/8*(c+d) - 1/16*(further neighbors)
            // Simplified version:
            [
                (pa[0] + pb[0]) * 0.5 + (pc[0] + pd_pt[0] - pa[0] - pb[0]) * 0.125,
                (pa[1] + pb[1]) * 0.5 + (pc[1] + pd_pt[1] - pa[1] - pb[1]) * 0.125,
                (pa[2] + pb[2]) * 0.5 + (pc[2] + pd_pt[2] - pa[2] - pb[2]) * 0.125,
            ]
        } else {
            [
                (pa[0] + pb[0]) * 0.5,
                (pa[1] + pb[1]) * 0.5,
                (pa[2] + pb[2]) * 0.5,
            ]
        }
    };

    let idx = points.len() as i64;
    points.push(mid_pt);
    cache.insert(key, idx);
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdivide_single_triangle() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = subdivide_butterfly(&pd);
        assert_eq!(result.points.len(), 6); // 3 + 3 midpoints
        assert_eq!(result.polys.num_cells(), 4); // 1 -> 4
    }

    #[test]
    fn subdivide_two_triangles() {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([1.0, 1.0, 0.0]);
        pd.points.push([0.0, 1.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 2, 3]);

        let result = subdivide_butterfly(&pd);
        assert_eq!(result.polys.num_cells(), 8); // 2 -> 8
                                                 // Shared edge midpoint should be reused
        assert!(result.points.len() < 12); // would be 12 if no sharing
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = subdivide_butterfly(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
