//! Boolean-based mesh splitting: split mesh by a plane into two halves.

use crate::data::{CellArray, Points, PolyData};

/// Split a mesh into two halves by a plane.
///
/// Returns (positive_side, negative_side).
pub fn split_by_plane(mesh: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> (PolyData, PolyData) {
    let n = mesh.points.len();
    let nlen = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
    if nlen < 1e-15 {
        return (mesh.clone(), PolyData::new());
    }
    let nn = [normal[0] / nlen, normal[1] / nlen, normal[2] / nlen];

    let signs: Vec<f64> = (0..n)
        .map(|i| {
            let p = mesh.points.get(i);
            (p[0] - origin[0]) * nn[0] + (p[1] - origin[1]) * nn[1] + (p[2] - origin[2]) * nn[2]
        })
        .collect();

    let (mut pos_pts, mut pos_polys) = (Points::<f64>::new(), CellArray::new());
    let (mut neg_pts, mut neg_polys) = (Points::<f64>::new(), CellArray::new());
    let mut pos_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut neg_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    for cell in mesh.polys.iter() {
        let all_pos = cell.iter().all(|&pid| signs[pid as usize] >= 0.0);
        let all_neg = cell.iter().all(|&pid| signs[pid as usize] < 0.0);

        if all_pos {
            let mut ids = Vec::new();
            for &pid in cell {
                let old = pid as usize;
                let idx = *pos_map.entry(old).or_insert_with(|| {
                    let i = pos_pts.len();
                    pos_pts.push(mesh.points.get(old));
                    i
                });
                ids.push(idx as i64);
            }
            pos_polys.push_cell(&ids);
        } else if all_neg {
            let mut ids = Vec::new();
            for &pid in cell {
                let old = pid as usize;
                let idx = *neg_map.entry(old).or_insert_with(|| {
                    let i = neg_pts.len();
                    neg_pts.push(mesh.points.get(old));
                    i
                });
                ids.push(idx as i64);
            }
            neg_polys.push_cell(&ids);
        }
        // Cells that straddle the plane are dropped (simple approach)
    }

    let mut pos = PolyData::new();
    pos.points = pos_pts;
    pos.polys = pos_polys;
    let mut neg = PolyData::new();
    neg.points = neg_pts;
    neg.polys = neg_polys;
    (pos, neg)
}

/// Split a mesh into N slabs along an axis.
pub fn split_into_slabs(mesh: &PolyData, axis: usize, n_slabs: usize) -> Vec<PolyData> {
    let n = mesh.points.len();
    if n == 0 || n_slabs == 0 {
        return Vec::new();
    }

    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;
    for i in 0..n {
        let v = mesh.points.get(i)[axis];
        min_v = min_v.min(v);
        max_v = max_v.max(v);
    }
    let range = (max_v - min_v).max(1e-15);
    let slab_width = range / n_slabs as f64;

    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();
    let mut slabs = Vec::with_capacity(n_slabs);

    for si in 0..n_slabs {
        let lo = min_v + si as f64 * slab_width;
        let hi = lo + slab_width;
        let mut pts = Points::<f64>::new();
        let mut polys = CellArray::new();
        let mut pt_map: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

        for cell in &all_cells {
            let centroid_v = cell
                .iter()
                .map(|&pid| mesh.points.get(pid as usize)[axis])
                .sum::<f64>()
                / cell.len() as f64;
            if centroid_v < lo || centroid_v >= hi {
                continue;
            }
            let mut ids = Vec::new();
            for &pid in cell {
                let old = pid as usize;
                let idx = *pt_map.entry(old).or_insert_with(|| {
                    let i = pts.len();
                    pts.push(mesh.points.get(old));
                    i
                });
                ids.push(idx as i64);
            }
            polys.push_cell(&ids);
        }

        let mut slab = PolyData::new();
        slab.points = pts;
        slab.polys = polys;
        slabs.push(slab);
    }
    slabs
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plane_split() {
        let mesh = PolyData::from_triangles(
            vec![
                [1.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [-3.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [-2.0, 2.0, 0.0],
            ],
            vec![[0, 1, 2], [3, 4, 5]],
        );
        let (pos, neg) = split_by_plane(&mesh, [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
        assert_eq!(pos.polys.num_cells(), 1); // first tri fully positive
        assert_eq!(neg.polys.num_cells(), 1); // second tri fully negative
    }
    #[test]
    fn slabs() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..10 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..4 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let slabs = split_into_slabs(&mesh, 0, 3);
        assert_eq!(slabs.len(), 3);
        let total: usize = slabs.iter().map(|s| s.polys.num_cells()).sum();
        assert_eq!(total, mesh.polys.num_cells());
    }
}
