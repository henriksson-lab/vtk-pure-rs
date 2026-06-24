use crate::data::{CellArray, Points, PolyData};
use std::collections::HashMap;

/// Flip edges to improve triangle quality (Delaunay-like criterion).
///
/// For each interior edge shared by two triangles, checks if flipping
/// the edge would improve the minimum angle. Performs one pass.
pub fn edge_flip_delaunay(input: &PolyData) -> PolyData {
    let n = input.points.len();
    let mut tris: Vec<[usize; 3]> = input
        .polys
        .iter()
        .filter(|c| c.len() == 3)
        .map(|c| [c[0] as usize, c[1] as usize, c[2] as usize])
        .collect();

    // Build edge-to-triangle adjacency
    let mut edge_tris: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (ti, tri) in tris.iter().enumerate() {
        for k in 0..3 {
            let a = tri[k];
            let b = tri[(k + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            edge_tris.entry(key).or_default().push(ti);
        }
    }

    let pts: Vec<[f64; 3]> = (0..n).map(|i| input.points.get(i)).collect();
    let mut flipped = true;
    let mut pass = 0;

    while flipped && pass < 5 {
        flipped = false;
        pass += 1;

        // Rebuild adjacency
        edge_tris.clear();
        for (ti, tri) in tris.iter().enumerate() {
            for k in 0..3 {
                let a = tri[k];
                let b = tri[(k + 1) % 3];
                let key = if a < b { (a, b) } else { (b, a) };
                edge_tris.entry(key).or_default().push(ti);
            }
        }

        let edges: Vec<(usize, usize)> = edge_tris.keys().copied().collect();
        for (a, b) in edges {
            let adj = match edge_tris.get(&(a, b)) {
                Some(v) if v.len() == 2 => (v[0], v[1]),
                _ => continue,
            };

            let t0 = tris[adj.0];
            let t1 = tris[adj.1];

            // Find opposite vertices
            let c = t0.iter().find(|&&v| v != a && v != b).copied();
            let d = t1.iter().find(|&&v| v != a && v != b).copied();
            let (c, d) = match (c, d) {
                (Some(c), Some(d)) => (c, d),
                _ => continue,
            };

            // Check if flip improves minimum angle
            let min_before = min_angle_pair(&pts, a, b, c, d);
            let min_after = min_angle_pair(&pts, c, d, a, b);

            if min_after > min_before + 0.01 {
                tris[adj.0] = [a, c, d];
                tris[adj.1] = [b, d, c];
                flipped = true;
            }
        }
    }

    let mut polys = CellArray::new();
    for tri in &tris {
        polys.push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
    }

    let mut pd = input.clone();
    pd.polys = polys;
    pd
}

fn min_angle_pair(pts: &[[f64; 3]], a: usize, b: usize, c: usize, d: usize) -> f64 {
    min_angle(pts[a], pts[b], pts[c]).min(min_angle(pts[a], pts[b], pts[d]))
}

fn min_angle(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let bc = [c[0] - b[0], c[1] - b[1], c[2] - b[2]];

    let angles = [
        angle_between(ab, ac),
        angle_between([-ab[0], -ab[1], -ab[2]], bc),
        angle_between([-ac[0], -ac[1], -ac[2]], [-bc[0], -bc[1], -bc[2]]),
    ];
    angles[0].min(angles[1]).min(angles[2])
}

fn angle_between(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let la = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    let lb = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
    if la < 1e-15 || lb < 1e-15 {
        return 0.0;
    }
    (dot / (la * lb)).clamp(-1.0, 1.0).acos()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_improves_quality() {
        // Two triangles sharing a long diagonal
        let mut pd = PolyData::new();
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([2.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.1, 0.0]); // nearly collinear
        pd.points.push([1.0, -0.1, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);
        pd.polys.push_cell(&[0, 3, 1]);

        let result = edge_flip_delaunay(&pd);
        assert_eq!(result.polys.num_cells(), 2);
    }

    #[test]
    fn already_good() {
        let mut pd = PolyData::new();
        let h = (3.0f64).sqrt() / 2.0;
        pd.points.push([0.0, 0.0, 0.0]);
        pd.points.push([1.0, 0.0, 0.0]);
        pd.points.push([0.5, h, 0.0]);
        pd.polys.push_cell(&[0, 1, 2]);

        let result = edge_flip_delaunay(&pd);
        assert_eq!(result.polys.num_cells(), 1);
    }

    #[test]
    fn empty_input() {
        let pd = PolyData::new();
        let result = edge_flip_delaunay(&pd);
        assert_eq!(result.polys.num_cells(), 0);
    }
}
