//! Adaptive subdivision: refine only where curvature or error exceeds threshold.

use crate::data::{CellArray, Points, PolyData};

/// Adaptively subdivide triangles where the edge length exceeds a threshold
/// OR where a predicate function returns true for the triangle centroid.
pub fn adaptive_subdivide_predicate(
    mesh: &PolyData,
    predicate: impl Fn([f64; 3], f64) -> bool, // (centroid, max_edge_len) -> should_split
    max_iterations: usize,
) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter_map(|c| {
            if c.len() == 3 {
                Some([c[0] as usize, c[1] as usize, c[2] as usize])
            } else {
                None
            }
        })
        .collect();

    for _ in 0..max_iterations {
        let mut new_tris = Vec::new();
        let mut changed = false;
        let mut edge_mids: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();

        for tri in &tris {
            let cx = (pts[tri[0]][0] + pts[tri[1]][0] + pts[tri[2]][0]) / 3.0;
            let cy = (pts[tri[0]][1] + pts[tri[1]][1] + pts[tri[2]][1]) / 3.0;
            let cz = (pts[tri[0]][2] + pts[tri[1]][2] + pts[tri[2]][2]) / 3.0;
            let max_edge = [
                elen(&pts, tri[0], tri[1]),
                elen(&pts, tri[1], tri[2]),
                elen(&pts, tri[2], tri[0]),
            ]
            .iter()
            .cloned()
            .fold(0.0f64, f64::max);

            if predicate([cx, cy, cz], max_edge) {
                // Subdivide: split all 3 edges
                let m01 = get_mid(&mut pts, &mut edge_mids, tri[0], tri[1]);
                let m12 = get_mid(&mut pts, &mut edge_mids, tri[1], tri[2]);
                let m20 = get_mid(&mut pts, &mut edge_mids, tri[2], tri[0]);
                new_tris.push([tri[0], m01, m20]);
                new_tris.push([tri[1], m12, m01]);
                new_tris.push([tri[2], m20, m12]);
                new_tris.push([m01, m12, m20]);
                changed = true;
            } else {
                new_tris.push(*tri);
            }
        }
        tris = new_tris;
        if !changed {
            break;
        }
    }

    let mut new_pts = Points::<f64>::new();
    for p in &pts {
        new_pts.push(*p);
    }
    let mut polys = CellArray::new();
    for t in &tris {
        polys.push_cell(&[t[0] as i64, t[1] as i64, t[2] as i64]);
    }
    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = polys;
    result
}

/// Subdivide only triangles with edge length above threshold.
pub fn subdivide_long_edges_adaptive(
    mesh: &PolyData,
    max_edge_length: f64,
    iterations: usize,
) -> PolyData {
    adaptive_subdivide_predicate(mesh, |_, max_e| max_e > max_edge_length, iterations)
}

/// Subdivide only triangles near a point (within radius).
pub fn subdivide_near_point(
    mesh: &PolyData,
    center: [f64; 3],
    radius: f64,
    iterations: usize,
) -> PolyData {
    let r2 = radius * radius;
    adaptive_subdivide_predicate(
        mesh,
        |c, _| {
            (c[0] - center[0]).powi(2) + (c[1] - center[1]).powi(2) + (c[2] - center[2]).powi(2)
                < r2
        },
        iterations,
    )
}

fn elen(pts: &[[f64; 3]], a: usize, b: usize) -> f64 {
    ((pts[a][0] - pts[b][0]).powi(2)
        + (pts[a][1] - pts[b][1]).powi(2)
        + (pts[a][2] - pts[b][2]).powi(2))
    .sqrt()
}

fn get_mid(
    pts: &mut Vec<[f64; 3]>,
    cache: &mut std::collections::HashMap<(usize, usize), usize>,
    a: usize,
    b: usize,
) -> usize {
    let key = (a.min(b), a.max(b));
    *cache.entry(key).or_insert_with(|| {
        let idx = pts.len();
        pts.push([
            (pts[a][0] + pts[b][0]) / 2.0,
            (pts[a][1] + pts[b][1]) / 2.0,
            (pts[a][2] + pts[b][2]) / 2.0,
        ]);
        idx
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn long_edge() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [5.0, 10.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = subdivide_long_edges_adaptive(&mesh, 3.0, 3);
        assert!(result.polys.num_cells() > 1);
    }
    #[test]
    fn near_point() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64 * 2.0, y as f64 * 2.0, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..4 {
                let bl = y * 5 + x;
                tris.push([bl, bl + 1, bl + 6]);
                tris.push([bl, bl + 6, bl + 5]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = subdivide_near_point(&mesh, [4.0, 4.0, 0.0], 3.0, 2);
        assert!(result.polys.num_cells() > mesh.polys.num_cells());
    }
    #[test]
    fn no_subdivision_needed() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [0.1, 0.0, 0.0], [0.0, 0.1, 0.0]],
            vec![[0, 1, 2]],
        );
        let result = subdivide_long_edges_adaptive(&mesh, 1.0, 3);
        assert_eq!(result.polys.num_cells(), 1); // already small enough
    }
}
