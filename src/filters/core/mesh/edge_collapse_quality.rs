//! Quality-aware edge collapse: preserve features and quality metrics.

use crate::data::{CellArray, Points, PolyData};

/// Collapse edges while preserving minimum triangle quality.
///
/// Only collapses edges where the resulting triangles maintain
/// quality above the threshold (aspect ratio < max_aspect_ratio).
pub fn quality_edge_collapse(
    mesh: &PolyData,
    target_faces: usize,
    max_aspect_ratio: f64,
) -> PolyData {
    let mut pts: Vec<[f64; 3]> = (0..mesh.points.len()).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = mesh
        .polys
        .iter()
        .filter_map(|c| {
            if c.len() == 3 {
                Some([
                    usize::try_from(c[0]).ok()?,
                    usize::try_from(c[1]).ok()?,
                    usize::try_from(c[2]).ok()?,
                ])
            } else {
                None
            }
        })
        .filter(|t| t.iter().all(|&id| id < pts.len()))
        .collect();
    let mut alive = vec![true; pts.len()];

    // Collect edges sorted by length
    let mut edges = collect_edges(&pts, &tris);
    edges.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    while tris.len() > target_faces {
        let mut collapsed = false;
        for &(a, b, _) in &edges {
            if !alive[a] || !alive[b] {
                continue;
            }

            // Simulate collapse: move a to midpoint, remove b
            let mid = [
                (pts[a][0] + pts[b][0]) / 2.0,
                (pts[a][1] + pts[b][1]) / 2.0,
                (pts[a][2] + pts[b][2]) / 2.0,
            ];
            let old_a = pts[a];

            // Check quality of affected triangles after collapse
            pts[a] = mid;
            let mut quality_ok = true;
            for t in &tris {
                let has_b = t[0] == b || t[1] == b || t[2] == b;
                let has_a = t[0] == a || t[1] == a || t[2] == a;
                if !has_a && !has_b {
                    continue;
                }
                // After collapse, b becomes a
                let mut new_tri = *t;
                for v in new_tri.iter_mut() {
                    if *v == b {
                        *v = a;
                    }
                }
                if new_tri[0] == new_tri[1] || new_tri[1] == new_tri[2] || new_tri[0] == new_tri[2]
                {
                    continue;
                } // degenerate, will be removed
                let ar = aspect_ratio(&pts, new_tri);
                if ar > max_aspect_ratio {
                    quality_ok = false;
                    break;
                }
            }

            if quality_ok {
                // Commit collapse
                alive[b] = false;
                for t in tris.iter_mut() {
                    for v in t.iter_mut() {
                        if *v == b {
                            *v = a;
                        }
                    }
                }
                tris.retain(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2]);
                collapsed = true;
                break;
            } else {
                pts[a] = old_a; // revert
            }
        }
        if !collapsed {
            break;
        }
        edges = collect_edges(&pts, &tris);
        edges.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    }

    // Build output
    let mut new_pts = Points::<f64>::new();
    let mut remap = vec![0usize; pts.len()];
    for i in 0..pts.len() {
        if alive[i] {
            remap[i] = new_pts.len();
            new_pts.push(pts[i]);
        }
    }
    let mut polys = CellArray::new();
    for t in &tris {
        if alive[t[0]] && alive[t[1]] && alive[t[2]] {
            polys.push_cell(&[remap[t[0]] as i64, remap[t[1]] as i64, remap[t[2]] as i64]);
        }
    }
    let mut result = PolyData::new();
    result.points = new_pts;
    result.polys = polys;
    result
}

fn collect_edges(pts: &[[f64; 3]], tris: &[[usize; 3]]) -> Vec<(usize, usize, f64)> {
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let mut edges = Vec::new();
    for t in tris {
        for i in 0..3 {
            let (a, b) = (t[i].min(t[(i + 1) % 3]), t[i].max(t[(i + 1) % 3]));
            if seen.insert((a, b)) {
                let len2 = (pts[a][0] - pts[b][0]).powi(2)
                    + (pts[a][1] - pts[b][1]).powi(2)
                    + (pts[a][2] - pts[b][2]).powi(2);
                edges.push((a, b, len2));
            }
        }
    }
    edges
}

fn aspect_ratio(pts: &[[f64; 3]], t: [usize; 3]) -> f64 {
    let lens = [
        ((pts[t[0]][0] - pts[t[1]][0]).powi(2)
            + (pts[t[0]][1] - pts[t[1]][1]).powi(2)
            + (pts[t[0]][2] - pts[t[1]][2]).powi(2))
        .sqrt(),
        ((pts[t[1]][0] - pts[t[2]][0]).powi(2)
            + (pts[t[1]][1] - pts[t[2]][1]).powi(2)
            + (pts[t[1]][2] - pts[t[2]][2]).powi(2))
        .sqrt(),
        ((pts[t[2]][0] - pts[t[0]][0]).powi(2)
            + (pts[t[2]][1] - pts[t[0]][1]).powi(2)
            + (pts[t[2]][2] - pts[t[0]][2]).powi(2))
        .sqrt(),
    ];
    let max_l = lens.iter().cloned().fold(0.0f64, f64::max);
    let min_l = lens.iter().cloned().fold(f64::MAX, f64::min);
    if min_l > 1e-15 {
        max_l / min_l
    } else {
        f64::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quality_collapse() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..6 {
            for x in 0..6 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..5 {
            for x in 0..5 {
                let bl = y * 6 + x;
                tris.push([bl, bl + 1, bl + 7]);
                tris.push([bl, bl + 7, bl + 6]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let result = quality_edge_collapse(&mesh, 20, 5.0);
        assert!(result.polys.num_cells() <= mesh.polys.num_cells());
        assert!(result.polys.num_cells() > 0);
    }
}
