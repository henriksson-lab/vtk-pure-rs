use crate::data::{CellArray, Points, PolyData};
use std::collections::{BTreeMap, BTreeSet};

/// Perform one iteration of Loop subdivision on a triangle mesh.
///
/// Each triangle is split into 4 by inserting edge midpoints. Original
/// vertices are repositioned using Loop's averaging rule:
///   new_pos = (1 - n*beta) * old_pos + beta * sum(neighbor_positions)
/// where n is the vertex valence and beta = 3/(8*n) for n > 3,
/// or 3/16 for n == 3.
///
/// Edge midpoints on interior edges use the Loop rule:
///   3/8 * (a + b) + 1/8 * (c + d)
/// where a,b are edge endpoints and c,d are the two opposite vertices.
/// Boundary edge midpoints use the simple midpoint.
pub fn subdivide_loop(input: &PolyData) -> PolyData {
    let num_orig_pts: usize = input.points.len();
    if input.polys.iter().any(|cell| cell.len() != 3) {
        return input.clone();
    }

    // Build adjacency: for each vertex, collect neighbor vertices
    let mut neighbors: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); num_orig_pts];
    let mut boundary_neighbors: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); num_orig_pts];

    // Edge to opposite vertices: edge (min,max) -> list of opposite vertex indices
    let mut edge_opposite: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();

    for cell in input.polys.iter() {
        let tri: [usize; 3] = [cell[0] as usize, cell[1] as usize, cell[2] as usize];

        for i in 0..3 {
            let a: usize = tri[i];
            let b: usize = tri[(i + 1) % 3];
            let opp: usize = tri[(i + 2) % 3];
            if a >= num_orig_pts || b >= num_orig_pts || opp >= num_orig_pts {
                return input.clone();
            }

            neighbors[a].insert(b);
            neighbors[b].insert(a);

            let key = if a < b { (a, b) } else { (b, a) };
            edge_opposite.entry(key).or_default().push(opp);
        }
    }
    if edge_opposite.values().any(|opposites| opposites.len() > 2) {
        return input.clone();
    }
    for (&(a, b), opposites) in &edge_opposite {
        if opposites.len() == 1 {
            boundary_neighbors[a].insert(b);
            boundary_neighbors[b].insert(a);
        }
    }

    // Compute new positions for original vertices (Loop's rule)
    let mut new_points: Points<f64> = Points::new();

    for i in 0..num_orig_pts {
        let p = input.points.get(i);
        let n: usize = neighbors[i].len();
        if n == 0 {
            new_points.push(p);
            continue;
        }
        if boundary_neighbors[i].len() >= 2 {
            let mut iter = boundary_neighbors[i].iter();
            let b0 = input.points.get(*iter.next().unwrap());
            let b1 = input.points.get(*iter.next().unwrap());
            new_points.push([
                0.75 * p[0] + 0.125 * (b0[0] + b1[0]),
                0.75 * p[1] + 0.125 * (b0[1] + b1[1]),
                0.75 * p[2] + 0.125 * (b0[2] + b1[2]),
            ]);
            continue;
        }

        let beta: f64 = if n == 3 {
            3.0 / 16.0
        } else {
            let cos_sq = 0.375 + 0.25 * (2.0 * std::f64::consts::PI / n as f64).cos();
            (0.625 - cos_sq * cos_sq) / n as f64
        };

        let mut sx: f64 = 0.0;
        let mut sy: f64 = 0.0;
        let mut sz: f64 = 0.0;
        for &nb in &neighbors[i] {
            let q = input.points.get(nb);
            sx += q[0];
            sy += q[1];
            sz += q[2];
        }

        let w: f64 = 1.0 - (n as f64) * beta;
        new_points.push([
            w * p[0] + beta * sx,
            w * p[1] + beta * sy,
            w * p[2] + beta * sz,
        ]);
    }

    // Create edge midpoints
    // Map edge (min,max) -> new point index
    let mut edge_point_map: BTreeMap<(usize, usize), usize> = BTreeMap::new();

    for (&(a, b), opposites) in &edge_opposite {
        let pa = input.points.get(a);
        let pb = input.points.get(b);

        let mid: [f64; 3];
        if opposites.len() == 2 {
            // Interior edge: Loop rule
            let pc = input.points.get(opposites[0]);
            let pd = input.points.get(opposites[1]);
            mid = [
                3.0 / 8.0 * (pa[0] + pb[0]) + 1.0 / 8.0 * (pc[0] + pd[0]),
                3.0 / 8.0 * (pa[1] + pb[1]) + 1.0 / 8.0 * (pc[1] + pd[1]),
                3.0 / 8.0 * (pa[2] + pb[2]) + 1.0 / 8.0 * (pc[2] + pd[2]),
            ];
        } else {
            // Boundary edge: simple midpoint
            mid = [
                0.5 * (pa[0] + pb[0]),
                0.5 * (pa[1] + pb[1]),
                0.5 * (pa[2] + pb[2]),
            ];
        }

        let idx: usize = new_points.len();
        new_points.push(mid);
        edge_point_map.insert((a, b), idx);
    }

    // Build new triangles: each original triangle -> 4 new triangles
    let mut new_polys = CellArray::new();

    for cell in input.polys.iter() {
        if cell.len() != 3 {
            continue;
        }
        let v0: usize = cell[0] as usize;
        let v1: usize = cell[1] as usize;
        let v2: usize = cell[2] as usize;

        let e01: usize = *edge_point_map
            .get(&if v0 < v1 { (v0, v1) } else { (v1, v0) })
            .unwrap();
        let e12: usize = *edge_point_map
            .get(&if v1 < v2 { (v1, v2) } else { (v2, v1) })
            .unwrap();
        let e20: usize = *edge_point_map
            .get(&if v2 < v0 { (v2, v0) } else { (v0, v2) })
            .unwrap();

        // 4 sub-triangles
        new_polys.push_cell(&[v0 as i64, e01 as i64, e20 as i64]);
        new_polys.push_cell(&[v1 as i64, e12 as i64, e01 as i64]);
        new_polys.push_cell(&[v2 as i64, e20 as i64, e12 as i64]);
        new_polys.push_cell(&[e01 as i64, e12 as i64, e20 as i64]);
    }

    let mut output = PolyData::new();
    output.points = new_points;
    output.polys = new_polys;
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_triangle_subdivides_to_four() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = subdivide_loop(&pd);
        // 3 original + 3 edge midpoints = 6 points
        assert_eq!(result.points.len(), 6);
        // 1 triangle -> 4 triangles
        assert_eq!(result.polys.num_cells(), 4);
    }

    #[test]
    fn two_triangles_shared_edge() {
        let pd = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [1.5, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [1, 3, 2]],
        );
        let result = subdivide_loop(&pd);
        // 4 original + 5 edges = 9 points
        assert_eq!(result.points.len(), 9);
        // 2 triangles -> 8 triangles
        assert_eq!(result.polys.num_cells(), 8);
    }

    #[test]
    fn all_triangles_are_valid() {
        let pd = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = subdivide_loop(&pd);
        for cell in result.polys.iter() {
            assert_eq!(cell.len(), 3, "All cells should be triangles");
            for &idx in cell {
                assert!(
                    (idx as usize) < result.points.len(),
                    "Point index out of bounds"
                );
            }
        }
    }
}
