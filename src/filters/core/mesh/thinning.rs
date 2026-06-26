use crate::data::{CellArray, Points, PolyData};

/// Mesh thinning: iteratively remove vertices with low connectivity
/// (valence) while preserving topology.
///
/// Removes vertices that are not boundary vertices and have the lowest
/// valence, collapsing their incident triangles, until the vertex count
/// reaches `target_vertices`. If `target_vertices >= current count`, the
/// mesh is returned unchanged.
pub fn thin_mesh(input: &PolyData, target_vertices: usize) -> PolyData {
    let n_pts = input.points.len();
    if n_pts <= target_vertices || n_pts < 4 {
        return input.clone();
    }

    // Copy points
    let points: Vec<[f64; 3]> = (0..n_pts).map(|i| input.points.get(i)).collect();

    // Copy triangles
    let mut triangles: Vec<Option<[usize; 3]>> = Vec::new();
    for cell in input.polys.iter() {
        if cell.len() == 3 && cell.iter().all(|&id| id >= 0 && (id as usize) < n_pts) {
            triangles.push(Some([cell[0] as usize, cell[1] as usize, cell[2] as usize]));
        }
    }

    let mut alive = vec![true; n_pts];
    let mut current_count: usize = n_pts;

    while current_count > target_vertices {
        // Build valence map and boundary detection
        let mut valence = vec![0usize; points.len()];
        let mut edge_count = std::collections::HashMap::<(usize, usize), usize>::new();

        for tri in triangles.iter().flatten() {
            for &v in tri {
                if alive[v] {
                    valence[v] += 1;
                }
            }
            let edges = [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])];
            for &(a, b) in &edges {
                let key = if a < b { (a, b) } else { (b, a) };
                *edge_count.entry(key).or_insert(0) += 1;
            }
        }

        // Boundary vertices: incident to an edge used by only 1 triangle
        let mut is_boundary = vec![false; points.len()];
        for (&(a, b), &count) in &edge_count {
            if count == 1 {
                is_boundary[a] = true;
                is_boundary[b] = true;
            }
        }

        // Find the vertex with lowest valence that is not a boundary vertex
        let mut best_v: Option<usize> = None;
        let mut best_val: usize = usize::MAX;

        for i in 0..points.len() {
            if alive[i] && !is_boundary[i] && valence[i] > 0 && valence[i] < best_val {
                best_val = valence[i];
                best_v = Some(i);
            }
        }

        let victim = match best_v {
            Some(v) => v,
            None => break, // no removable vertices left
        };

        // Find a neighbor to collapse onto
        let mut collapse_target: Option<usize> = None;
        for tri in triangles.iter().flatten() {
            if tri[0] == victim || tri[1] == victim || tri[2] == victim {
                for &v in tri {
                    if v != victim && alive[v] {
                        collapse_target = Some(v);
                        break;
                    }
                }
                if collapse_target.is_some() {
                    break;
                }
            }
        }

        let target = match collapse_target {
            Some(t) => t,
            None => break,
        };

        // Remove victim: remap all references to victim -> target
        alive[victim] = false;
        current_count -= 1;

        for tri in &mut triangles {
            if let Some(ref mut t) = tri {
                for v in t.iter_mut() {
                    if *v == victim {
                        *v = target;
                    }
                }
                // Remove degenerate triangles (two or more identical vertices)
                if t[0] == t[1] || t[1] == t[2] || t[0] == t[2] {
                    *tri = None;
                }
            }
        }
    }

    // Build output
    let mut new_points = Points::<f64>::new();
    let mut remap = vec![0usize; points.len()];
    let mut new_idx: usize = 0;

    for i in 0..points.len() {
        if alive[i] {
            new_points.push(points[i]);
            remap[i] = new_idx;
            new_idx += 1;
        }
    }

    let mut new_polys = CellArray::new();
    for tri in triangles.iter().flatten() {
        if alive[tri[0]] && alive[tri[1]] && alive[tri[2]] {
            new_polys.push_cell(&[
                remap[tri[0]] as i64,
                remap[tri[1]] as i64,
                remap[tri[2]] as i64,
            ]);
        }
    }

    let mut output = PolyData::new();
    output.points = new_points;
    output.polys = new_polys;
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_grid_mesh() -> PolyData {
        // 4x4 grid = 16 vertices, 18 triangles; 4 interior vertices removable
        let mut pts = Vec::new();
        for j in 0..4 {
            for i in 0..4 {
                pts.push([i as f64, j as f64, 0.0]);
            }
        }
        let mut tris = Vec::new();
        for j in 0..3 {
            for i in 0..3 {
                let v0 = j * 4 + i;
                let v1 = v0 + 1;
                let v2 = v0 + 4;
                let v3 = v2 + 1;
                tris.push([v0, v1, v3]);
                tris.push([v0, v3, v2]);
            }
        }
        PolyData::from_triangles(pts, tris)
    }

    #[test]
    fn reduces_vertex_count() {
        let pd = make_grid_mesh();
        assert_eq!(pd.points.len(), 16);
        // 4 interior vertices can be removed; ask to go down to 14
        let result = thin_mesh(&pd, 14);
        assert!(
            result.points.len() <= 14,
            "expected <= 14 vertices, got {}",
            result.points.len()
        );
    }

    #[test]
    fn target_above_current_unchanged() {
        let pd = make_grid_mesh();
        let result = thin_mesh(&pd, 100);
        assert_eq!(result.points.len(), pd.points.len());
    }

    #[test]
    fn no_degenerate_triangles() {
        let pd = make_grid_mesh();
        let result = thin_mesh(&pd, 5);
        for cell in result.polys.iter() {
            assert_eq!(cell.len(), 3);
            assert_ne!(cell[0], cell[1]);
            assert_ne!(cell[1], cell[2]);
            assert_ne!(cell[0], cell[2]);
        }
    }
}
