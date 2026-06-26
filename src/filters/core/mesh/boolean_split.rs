//! Boolean-based mesh splitting: split mesh by a plane into two halves.

use crate::data::{CellArray, Points, PolyData};

/// Split a mesh into two halves by a plane.
///
/// Returns (positive_side, negative_side).
pub fn split_by_plane(mesh: &PolyData, origin: [f64; 3], normal: [f64; 3]) -> (PolyData, PolyData) {
    let nlen = (normal[0].powi(2) + normal[1].powi(2) + normal[2].powi(2)).sqrt();
    if nlen < 1e-15 {
        return (mesh.clone(), PolyData::new());
    }
    let nn = [normal[0] / nlen, normal[1] / nlen, normal[2] / nlen];

    let (mut pos_pts, mut pos_polys) = (Points::<f64>::new(), CellArray::new());
    let (mut neg_pts, mut neg_polys) = (Points::<f64>::new(), CellArray::new());

    for cell in mesh.polys.iter() {
        if cell.len() < 3 {
            continue;
        }

        let poly: Vec<([f64; 3], f64)> = cell
            .iter()
            .map(|&pid| {
                let p = mesh.points.get(pid as usize);
                (p, signed_distance(p, origin, nn))
            })
            .collect();

        let pos_poly = clip_polygon(&poly, true);
        if pos_poly.len() >= 3 {
            push_polygon(&pos_poly, &mut pos_pts, &mut pos_polys);
        }

        let neg_poly = clip_polygon(&poly, false);
        if neg_poly.len() >= 3 {
            push_polygon(&neg_poly, &mut neg_pts, &mut neg_polys);
        }
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
    if n == 0 || n_slabs == 0 || axis >= 3 {
        return Vec::new();
    }

    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
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
            if centroid_v < lo
                || (si + 1 < n_slabs && centroid_v >= hi)
                || (si + 1 == n_slabs && centroid_v > hi)
            {
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

fn signed_distance(p: [f64; 3], origin: [f64; 3], normal: [f64; 3]) -> f64 {
    (p[0] - origin[0]) * normal[0] + (p[1] - origin[1]) * normal[1] + (p[2] - origin[2]) * normal[2]
}

fn clip_polygon(poly: &[([f64; 3], f64)], keep_positive: bool) -> Vec<[f64; 3]> {
    let mut out = Vec::new();
    if poly.is_empty() {
        return out;
    }

    let inside = |d: f64| {
        if keep_positive {
            d >= -1e-12
        } else {
            d <= 1e-12
        }
    };
    let mut prev = *poly.last().unwrap();
    let mut prev_inside = inside(prev.1);

    for &cur in poly {
        let cur_inside = inside(cur.1);
        if cur_inside != prev_inside {
            out.push(intersect_edge(prev, cur));
        }
        if cur_inside {
            out.push(cur.0);
        }
        prev = cur;
        prev_inside = cur_inside;
    }

    dedup_consecutive(out)
}

fn intersect_edge(a: ([f64; 3], f64), b: ([f64; 3], f64)) -> [f64; 3] {
    let denom = a.1 - b.1;
    let t = if denom.abs() < 1e-15 {
        0.5
    } else {
        a.1 / denom
    };
    [
        a.0[0] + t * (b.0[0] - a.0[0]),
        a.0[1] + t * (b.0[1] - a.0[1]),
        a.0[2] + t * (b.0[2] - a.0[2]),
    ]
}

fn dedup_consecutive(points: Vec<[f64; 3]>) -> Vec<[f64; 3]> {
    let mut out = Vec::new();
    for p in points {
        if out.last().map_or(true, |q| distance2(*q, p) > 1e-24) {
            out.push(p);
        }
    }
    if out.len() > 1 && distance2(out[0], *out.last().unwrap()) <= 1e-24 {
        out.pop();
    }
    out
}

fn distance2(a: [f64; 3], b: [f64; 3]) -> f64 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

fn push_polygon(poly: &[[f64; 3]], points: &mut Points<f64>, polys: &mut CellArray) {
    let base = points.len() as i64;
    for &p in poly {
        points.push(p);
    }
    let ids: Vec<i64> = (0..poly.len()).map(|i| base + i as i64).collect();
    polys.push_cell(&ids);
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
