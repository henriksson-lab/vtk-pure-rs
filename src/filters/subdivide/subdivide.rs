use std::collections::HashMap;

use crate::data::{CellArray, Points, PolyData};

/// Subdivide a triangle mesh using Loop subdivision.
///
/// Each triangle is split into 4 sub-triangles. Edge midpoints are computed as
/// weighted averages of the edge endpoints and opposite vertices (for interior edges)
/// or simple midpoints (for boundary edges). Existing vertices are repositioned
/// using their valence-weighted neighbors.
///
/// Input must be a triangle mesh. Run `triangulate` first if needed.
pub fn subdivide(input: &PolyData, iterations: usize) -> PolyData {
    let mut current = input.clone();

    for _ in 0..iterations {
        current = subdivide_once(&current);
    }

    current
}

fn subdivide_once(input: &PolyData) -> PolyData {
    let n_pts = input.points.len();
    let mut new_points = Points::<f64>::new();

    // Copy original points (they'll be repositioned later)
    for i in 0..n_pts {
        new_points.push(input.points.get(i));
    }

    // Map from (min_idx, max_idx) edge -> new midpoint index
    let mut edge_midpoints: HashMap<(usize, usize), usize> = HashMap::new();

    // Collect edge-to-triangle adjacency for weighting
    let mut edge_triangles: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    let mut triangles: Vec<[usize; 3]> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let tri_idx = triangles.len();
        let t = [cell[0] as usize, cell[1] as usize, cell[2] as usize];
        triangles.push(t);
        for j in 0..3 {
            let a = t[j];
            let b = t[(j + 1) % 3];
            let edge = if a < b { (a, b) } else { (b, a) };
            edge_triangles.entry(edge).or_default().push(tri_idx);
        }
    }

    // Create midpoints for each edge
    for (&(a, b), tris) in &edge_triangles {
        if edge_midpoints.contains_key(&(a, b)) {
            continue;
        }

        let pa = input.points.get(a);
        let pb = input.points.get(b);

        let mid = if tris.len() == 2 {
            // Interior edge: 3/8 * (a + b) + 1/8 * (c + d) where c,d are opposite vertices
            let c = opposite_vertex(&triangles[tris[0]], a, b);
            let d = opposite_vertex(&triangles[tris[1]], a, b);
            let pc = input.points.get(c);
            let pd = input.points.get(d);
            [
                0.375 * (pa[0] + pb[0]) + 0.125 * (pc[0] + pd[0]),
                0.375 * (pa[1] + pb[1]) + 0.125 * (pc[1] + pd[1]),
                0.375 * (pa[2] + pb[2]) + 0.125 * (pc[2] + pd[2]),
            ]
        } else {
            // Boundary edge: simple midpoint
            [
                0.5 * (pa[0] + pb[0]),
                0.5 * (pa[1] + pb[1]),
                0.5 * (pa[2] + pb[2]),
            ]
        };

        let idx = new_points.len();
        new_points.push(mid);
        edge_midpoints.insert((a, b), idx);
    }

    // Reposition original vertices
    let mut neighbors: Vec<Vec<usize>> = vec![Vec::new(); n_pts];
    for &(a, b) in edge_triangles.keys() {
        if !neighbors[a].contains(&b) {
            neighbors[a].push(b);
        }
        if !neighbors[b].contains(&a) {
            neighbors[b].push(a);
        }
    }

    // Check which vertices are on the boundary
    let mut is_boundary = vec![false; n_pts];
    for (&(_a, _b), tris) in &edge_triangles {
        if tris.len() == 1 {
            is_boundary[_a] = true;
            is_boundary[_b] = true;
        }
    }

    for i in 0..n_pts {
        let p = input.points.get(i);
        let n = neighbors[i].len();
        if n == 0 {
            continue;
        }

        if is_boundary[i] {
            // Boundary vertex: keep in place (or use boundary rule)
            // Simple: keep original position
            continue;
        }

        // Interior vertex: Loop subdivision weight
        let beta = if n == 3 {
            3.0 / 16.0
        } else {
            3.0 / (8.0 * n as f64)
        };

        let mut avg = [0.0f64; 3];
        for &nb in &neighbors[i] {
            let q = input.points.get(nb);
            avg[0] += q[0];
            avg[1] += q[1];
            avg[2] += q[2];
        }

        let new_pos = [
            (1.0 - n as f64 * beta) * p[0] + beta * avg[0],
            (1.0 - n as f64 * beta) * p[1] + beta * avg[1],
            (1.0 - n as f64 * beta) * p[2] + beta * avg[2],
        ];
        new_points.set(i, new_pos);
    }

    // Generate 4 sub-triangles per original triangle
    let mut polys = CellArray::new();
    for tri in &triangles {
        let [v0, v1, v2] = *tri;

        let m01 = *edge_midpoints.get(&edge_key(v0, v1)).unwrap();
        let m12 = *edge_midpoints.get(&edge_key(v1, v2)).unwrap();
        let m20 = *edge_midpoints.get(&edge_key(v2, v0)).unwrap();

        polys.push_cell(&[v0 as i64, m01 as i64, m20 as i64]);
        polys.push_cell(&[v1 as i64, m12 as i64, m01 as i64]);
        polys.push_cell(&[v2 as i64, m20 as i64, m12 as i64]);
        polys.push_cell(&[m01 as i64, m12 as i64, m20 as i64]);
    }

    let mut output = PolyData::new();
    output.points = new_points;
    output.polys = polys;
    output
}

fn edge_key(a: usize, b: usize) -> (usize, usize) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

fn opposite_vertex(tri: &[usize; 3], a: usize, b: usize) -> usize {
    for &v in tri {
        if v != a && v != b {
            return v;
        }
    }
    tri[0] // fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdivide_single_triangle() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = subdivide(&pd, 1);
        // 1 triangle → 4 triangles, 3 original + 3 midpoints = 6 points
        assert_eq!(result.polys.num_cells(), 4);
        assert_eq!(result.points.len(), 6);
    }

    #[test]
    fn subdivide_two_iterations() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );

        let result = subdivide(&pd, 2);
        // 1 → 4 → 16 triangles
        assert_eq!(result.polys.num_cells(), 16);
    }

    #[test]
    fn subdivide_preserves_manifold() {
        // Two triangles sharing an edge
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, -1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 3, 1]],
        );

        let result = subdivide(&pd, 1);
        // 2 triangles → 8
        assert_eq!(result.polys.num_cells(), 8);
    }
}
