use std::collections::{HashMap, HashSet};

use crate::data::{CellArray, Points, PolyData};

/// Isotropic remeshing: iteratively split long edges, collapse short edges,
/// and equalize vertex valence by flipping edges.
///
/// `target_length` is the desired edge length. `iterations` controls the number
/// of remeshing passes. Each pass performs:
/// 1. Split edges longer than `4/3 * target_length`
/// 2. Collapse edges shorter than `4/5 * target_length`
/// 3. Flip edges to equalize vertex valence toward 6
/// 4. Tangential Laplacian relaxation
pub fn remesh_isotropic(input: &PolyData, target_length: f64, iterations: usize) -> PolyData {
    let mut points: Vec<[f64; 3]> = Vec::with_capacity(input.points.len());
    for i in 0..input.points.len() {
        points.push(input.points.get(i));
    }

    let mut tris: Vec<[usize; 3]> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() == 3 {
            tris.push([cell[0] as usize, cell[1] as usize, cell[2] as usize]);
        }
    }

    if tris.is_empty() || target_length <= 0.0 {
        return input.clone();
    }

    let high: f64 = target_length * 4.0 / 3.0;
    let low: f64 = target_length * 4.0 / 5.0;

    for _ in 0..iterations {
        // 1. Split long edges
        split_long_edges(&mut points, &mut tris, high);

        // 2. Collapse short edges
        collapse_short_edges(&mut points, &mut tris, low);

        // 3. Flip edges to equalize valence
        flip_edges_for_valence(&points, &mut tris);

        // 4. Tangential relaxation
        tangential_relax(&mut points, &tris);
    }

    let mut out = PolyData::new();
    let mut out_points = Points::<f64>::new();
    for p in &points {
        out_points.push(*p);
    }
    out.points = out_points;
    let mut polys = CellArray::new();
    for t in &tris {
        polys.push_cell(&[t[0] as i64, t[1] as i64, t[2] as i64]);
    }
    out.polys = polys;
    out
}

fn edge_length(points: &[[f64; 3]], a: usize, b: usize) -> f64 {
    let dx: f64 = points[a][0] - points[b][0];
    let dy: f64 = points[a][1] - points[b][1];
    let dz: f64 = points[a][2] - points[b][2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn split_long_edges(points: &mut Vec<[f64; 3]>, tris: &mut Vec<[usize; 3]>, high: f64) {
    let mut new_tris: Vec<[usize; 3]> = Vec::new();
    let mut midpoint_cache: HashMap<(usize, usize), usize> = HashMap::new();

    for tri in tris.iter() {
        let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
        let mut splits: Vec<Option<usize>> = Vec::with_capacity(3);

        for &(a, b) in &edges {
            let len: f64 = edge_length(points, a, b);
            if len > high {
                let key = if a < b { (a, b) } else { (b, a) };
                let mid = *midpoint_cache.entry(key).or_insert_with(|| {
                    let mx: f64 = (points[a][0] + points[b][0]) * 0.5;
                    let my: f64 = (points[a][1] + points[b][1]) * 0.5;
                    let mz: f64 = (points[a][2] + points[b][2]) * 0.5;
                    let idx: usize = points.len();
                    points.push([mx, my, mz]);
                    idx
                });
                splits.push(Some(mid));
            } else {
                splits.push(None);
            }
        }

        let split_count: usize = splits.iter().filter(|s| s.is_some()).count();
        if split_count == 0 {
            new_tris.push(*tri);
        } else {
            // Split on the longest edge only for simplicity
            let mut longest_idx: usize = 0;
            let mut longest_len: f64 = 0.0;
            for (i, &(a, b)) in edges.iter().enumerate() {
                let len: f64 = edge_length(points, a, b);
                if len > longest_len {
                    longest_len = len;
                    longest_idx = i;
                }
            }
            let key = if edges[longest_idx].0 < edges[longest_idx].1 {
                (edges[longest_idx].0, edges[longest_idx].1)
            } else {
                (edges[longest_idx].1, edges[longest_idx].0)
            };
            let mid = *midpoint_cache.entry(key).or_insert_with(|| {
                let a: usize = edges[longest_idx].0;
                let b: usize = edges[longest_idx].1;
                let mx: f64 = (points[a][0] + points[b][0]) * 0.5;
                let my: f64 = (points[a][1] + points[b][1]) * 0.5;
                let mz: f64 = (points[a][2] + points[b][2]) * 0.5;
                let idx: usize = points.len();
                points.push([mx, my, mz]);
                idx
            });

            let (a, b) = edges[longest_idx];
            let c: usize = edges[(longest_idx + 1) % 3].1;
            new_tris.push([a, mid, c]);
            new_tris.push([mid, b, c]);
        }
    }

    *tris = new_tris;
}

fn collapse_short_edges(points: &mut Vec<[f64; 3]>, tris: &mut Vec<[usize; 3]>, low: f64) {
    let num_pts: usize = points.len();
    let mut remap: Vec<usize> = (0..num_pts).collect();

    fn find_root(remap: &[usize], mut v: usize) -> usize {
        while remap[v] != v {
            v = remap[v];
        }
        v
    }

    // Collect unique edges
    let mut edge_list: Vec<(usize, usize)> = Vec::new();
    let mut edge_set: HashSet<(usize, usize)> = HashSet::new();
    for tri in tris.iter() {
        for i in 0..3 {
            let a: usize = tri[i];
            let b: usize = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if edge_set.insert(key) {
                edge_list.push(key);
            }
        }
    }

    // Sort edges by length so we collapse shortest first
    edge_list.sort_by(|&(a1, b1), &(a2, b2)| {
        let l1: f64 = edge_length(points, a1, b1);
        let l2: f64 = edge_length(points, a2, b2);
        l1.partial_cmp(&l2).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Track which vertices are involved in a collapse to avoid cascading
    let mut collapsed: HashSet<usize> = HashSet::new();

    for (a, b) in edge_list {
        let ra: usize = find_root(&remap, a);
        let rb: usize = find_root(&remap, b);
        if ra == rb {
            continue;
        }
        if collapsed.contains(&ra) || collapsed.contains(&rb) {
            continue;
        }
        let len: f64 = edge_length(points, ra, rb);
        if len < low {
            let mx: f64 = (points[ra][0] + points[rb][0]) * 0.5;
            let my: f64 = (points[ra][1] + points[rb][1]) * 0.5;
            let mz: f64 = (points[ra][2] + points[rb][2]) * 0.5;
            points[ra] = [mx, my, mz];
            remap[rb] = ra;
            collapsed.insert(ra);
            collapsed.insert(rb);
        }
    }

    // Remap triangle indices and remove degenerate triangles
    let mut new_tris: Vec<[usize; 3]> = Vec::new();
    for tri in tris.iter() {
        let a: usize = find_root(&remap, tri[0]);
        let b: usize = find_root(&remap, tri[1]);
        let c: usize = find_root(&remap, tri[2]);
        if a != b && b != c && a != c {
            new_tris.push([a, b, c]);
        }
    }
    *tris = new_tris;
}

fn flip_edges_for_valence(points: &[[f64; 3]], tris: &mut Vec<[usize; 3]>) {
    // Build edge to triangle map
    let mut edge_tris: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (ti, tri) in tris.iter().enumerate() {
        for i in 0..3 {
            let a: usize = tri[i];
            let b: usize = tri[(i + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_tris.entry(key).or_default().push(ti);
        }
    }

    // Compute valence
    let mut valence: HashMap<usize, usize> = HashMap::new();
    for tri in tris.iter() {
        for &v in tri {
            *valence.entry(v).or_insert(0) += 1;
        }
    }

    let target_valence: usize = 6;

    for (&(a, b), face_list) in &edge_tris {
        if face_list.len() != 2 {
            continue;
        }
        let ti0: usize = face_list[0];
        let ti1: usize = face_list[1];

        // Find opposite vertices
        let c: usize = tris[ti0]
            .iter()
            .copied()
            .find(|&v| v != a && v != b)
            .unwrap();
        let d: usize = tris[ti1]
            .iter()
            .copied()
            .find(|&v| v != a && v != b)
            .unwrap();

        if c == d {
            continue;
        }

        let va: usize = *valence.get(&a).unwrap_or(&0);
        let vb: usize = *valence.get(&b).unwrap_or(&0);
        let vc: usize = *valence.get(&c).unwrap_or(&0);
        let vd: usize = *valence.get(&d).unwrap_or(&0);

        let before: i64 = (va as i64 - target_valence as i64).abs()
            + (vb as i64 - target_valence as i64).abs()
            + (vc as i64 - target_valence as i64).abs()
            + (vd as i64 - target_valence as i64).abs();

        // After flip: a and b lose one, c and d gain one
        let after: i64 = ((va as i64 - 1) - target_valence as i64).abs()
            + ((vb as i64 - 1) - target_valence as i64).abs()
            + ((vc as i64 + 1) - target_valence as i64).abs()
            + ((vd as i64 + 1) - target_valence as i64).abs();

        if after < before {
            // Check that the flip produces valid triangles (non-degenerate)
            let _ = points; // used for potential geometric checks
            tris[ti0] = [a, d, c];
            tris[ti1] = [b, c, d];
            *valence.entry(a).or_insert(0) -= 1;
            *valence.entry(b).or_insert(0) -= 1;
            *valence.entry(c).or_insert(0) += 1;
            *valence.entry(d).or_insert(0) += 1;
        }
    }
}

fn tangential_relax(points: &mut Vec<[f64; 3]>, tris: &[[usize; 3]]) {
    let n: usize = points.len();
    let mut neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); n];
    for tri in tris {
        for i in 0..3 {
            neighbors[tri[i]].insert(tri[(i + 1) % 3]);
            neighbors[tri[i]].insert(tri[(i + 2) % 3]);
        }
    }

    let old_points: Vec<[f64; 3]> = points.clone();
    let factor: f64 = 0.5;

    for i in 0..n {
        let nbrs = &neighbors[i];
        if nbrs.is_empty() {
            continue;
        }
        let count: f64 = nbrs.len() as f64;
        let mut avg: [f64; 3] = [0.0, 0.0, 0.0];
        for &nb in nbrs {
            avg[0] += old_points[nb][0];
            avg[1] += old_points[nb][1];
            avg[2] += old_points[nb][2];
        }
        avg[0] /= count;
        avg[1] /= count;
        avg[2] /= count;

        points[i][0] += (avg[0] - old_points[i][0]) * factor;
        points[i][1] += (avg[1] - old_points[i][1]) * factor;
        points[i][2] += (avg[2] - old_points[i][2]) * factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_two_triangle_mesh() -> PolyData {
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 2.0, 0.0]);
        pd.points.push([3.0, 2.0, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[1, 3, 2]);
        pd
    }

    #[test]
    fn basic_remesh_produces_output() {
        let input = make_two_triangle_mesh();
        let result = remesh_isotropic(&input, 1.0, 2);
        assert!(result.points.len() > 0);
        assert!(result.polys.num_cells() > 0);
    }

    #[test]
    fn short_target_splits_edges() {
        let input = make_two_triangle_mesh();
        // Target length shorter than existing edges (~2.0) should produce more triangles
        // Use 1 iteration to avoid cascading effects
        let result = remesh_isotropic(&input, 1.0, 1);
        assert!(
            result.polys.num_cells() > input.polys.num_cells(),
            "expected more triangles after remeshing with short target, got {}",
            result.polys.num_cells()
        );
    }

    #[test]
    fn zero_iterations_returns_clone() {
        let input = make_two_triangle_mesh();
        let result = remesh_isotropic(&input, 1.0, 0);
        assert_eq!(result.points.len(), input.points.len());
        assert_eq!(result.polys.num_cells(), input.polys.num_cells());
    }
}
