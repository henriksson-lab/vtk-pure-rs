//! Progressive mesh decimation with LOD (level of detail) output.

use crate::data::{CellArray, Points, PolyData};

/// Generate multiple LOD meshes by progressive decimation.
///
/// Returns a vector of meshes from finest (original) to coarsest.
pub fn generate_lod_meshes(mesh: &PolyData, n_levels: usize) -> Vec<PolyData> {
    let mut levels = Vec::with_capacity(n_levels);
    levels.push(mesh.clone());

    let mut current = mesh.clone();
    for level in 1..n_levels {
        let target_ratio = 1.0 - (level as f64 / n_levels as f64);
        current = decimate_to_ratio(&current, target_ratio);
        if current.polys.num_cells() == 0 {
            break;
        }
        levels.push(current.clone());
    }
    levels
}

/// Select appropriate LOD based on distance from camera.
pub fn select_lod(lods: &[PolyData], distance: f64, max_distance: f64) -> &PolyData {
    if lods.is_empty() {
        panic!("no LOD levels");
    }
    let t = (distance / max_distance).clamp(0.0, 1.0);
    let idx = (t * (lods.len() - 1) as f64) as usize;
    &lods[idx.min(lods.len() - 1)]
}

fn decimate_to_ratio(mesh: &PolyData, target_ratio: f64) -> PolyData {
    let n_pts = mesh.points.len();
    let n_cells = mesh.polys.num_cells();
    if n_pts < 4 || n_cells < 2 {
        return mesh.clone();
    }

    let target = (n_cells as f64 * target_ratio).max(1.0) as usize;
    let all_cells: Vec<Vec<i64>> = mesh.polys.iter().map(|c| c.to_vec()).collect();

    // Simple vertex-pair collapse
    let mut positions: Vec<[f64; 3]> = (0..n_pts).map(|i| mesh.points.get(i)).collect();
    let mut tris: Vec<[usize; 3]> = all_cells
        .iter()
        .filter_map(|c| {
            if c.len() == 3 {
                Some([c[0] as usize, c[1] as usize, c[2] as usize])
            } else {
                None
            }
        })
        .collect();
    let mut alive = vec![true; n_pts];

    // Collect edges sorted by length
    let mut edges: Vec<(usize, usize, f64)> = Vec::new();
    let mut seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    for t in &tris {
        for i in 0..3 {
            let (a, b) = (t[i].min(t[(i + 1) % 3]), t[i].max(t[(i + 1) % 3]));
            if seen.insert((a, b)) {
                let d = edge_len(&positions, a, b);
                edges.push((a, b, d));
            }
        }
    }
    edges.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    let mut current_count = tris.len();
    for &(a, b, _) in &edges {
        if current_count <= target {
            break;
        }
        if !alive[a] || !alive[b] {
            continue;
        }
        positions[a] = midpoint(&positions[a], &positions[b]);
        alive[b] = false;
        for t in tris.iter_mut() {
            for v in t.iter_mut() {
                if *v == b {
                    *v = a;
                }
            }
        }
        let before = current_count;
        tris.retain(|t| t[0] != t[1] && t[1] != t[2] && t[0] != t[2]);
        current_count = tris.len();
    }

    let mut new_pts = Points::<f64>::new();
    let mut remap = vec![0usize; n_pts];
    for i in 0..n_pts {
        if alive[i] {
            remap[i] = new_pts.len();
            new_pts.push(positions[i]);
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

fn edge_len(pts: &[[f64; 3]], a: usize, b: usize) -> f64 {
    ((pts[a][0] - pts[b][0]).powi(2)
        + (pts[a][1] - pts[b][1]).powi(2)
        + (pts[a][2] - pts[b][2]).powi(2))
    .sqrt()
}
fn midpoint(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        (a[0] + b[0]) / 2.0,
        (a[1] + b[1]) / 2.0,
        (a[2] + b[2]) / 2.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_3_levels() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..10 {
            for x in 0..10 {
                pts.push([x as f64, y as f64, 0.0]);
            }
        }
        for y in 0..9 {
            for x in 0..9 {
                let bl = y * 10 + x;
                tris.push([bl, bl + 1, bl + 11]);
                tris.push([bl, bl + 11, bl + 10]);
            }
        }
        let mesh = PolyData::from_triangles(pts, tris);
        let lods = generate_lod_meshes(&mesh, 3);
        assert_eq!(lods.len(), 3);
        assert!(lods[1].polys.num_cells() < lods[0].polys.num_cells());
        assert!(lods[2].polys.num_cells() < lods[1].polys.num_cells());
    }
    #[test]
    fn select() {
        let mut pts = Vec::new();
        let mut tris = Vec::new();
        for y in 0..5 {
            for x in 0..5 {
                pts.push([x as f64, y as f64, 0.0]);
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
        let lods = generate_lod_meshes(&mesh, 3);
        let close = select_lod(&lods, 1.0, 100.0);
        let far = select_lod(&lods, 90.0, 100.0);
        assert!(close.polys.num_cells() >= far.polys.num_cells());
    }
}
