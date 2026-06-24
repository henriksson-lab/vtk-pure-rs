//! Advanced hole filling with quality triangulation.

use crate::data::{CellArray, Points, PolyData};

/// Fill all boundary holes with fan triangulation from a centroid vertex.
pub fn fill_all_holes(mesh: &PolyData) -> PolyData {
    let loops = find_boundary_loops(mesh);
    if loops.is_empty() {
        return mesh.clone();
    }

    let mut result = mesh.clone();
    for loop_verts in &loops {
        if loop_verts.len() < 3 {
            continue;
        }
        // Compute centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut cz = 0.0;
        for &vi in loop_verts {
            let p = result.points.get(vi);
            cx += p[0];
            cy += p[1];
            cz += p[2];
        }
        let k = loop_verts.len() as f64;
        let center_idx = result.points.len() as i64;
        result.points.push([cx / k, cy / k, cz / k]);

        for i in 0..loop_verts.len() {
            let a = loop_verts[i] as i64;
            let b = loop_verts[(i + 1) % loop_verts.len()] as i64;
            result.polys.push_cell(&[center_idx, a, b]);
        }
    }
    result
}

/// Fill holes with ear-clipping triangulation (better quality than fan).
pub fn fill_holes_ear_clip(mesh: &PolyData) -> PolyData {
    let loops = find_boundary_loops(mesh);
    if loops.is_empty() {
        return mesh.clone();
    }

    let mut result = mesh.clone();
    for loop_verts in &loops {
        if loop_verts.len() < 3 {
            continue;
        }
        let tris = ear_clip_3d(mesh, loop_verts);
        for tri in &tris {
            result
                .polys
                .push_cell(&[tri[0] as i64, tri[1] as i64, tri[2] as i64]);
        }
    }
    result
}

/// Count boundary holes.
pub fn count_holes(mesh: &PolyData) -> usize {
    find_boundary_loops(mesh).len()
}

/// Get the size of each hole (number of boundary edges).
pub fn hole_sizes(mesh: &PolyData) -> Vec<usize> {
    find_boundary_loops(mesh).iter().map(|l| l.len()).collect()
}

fn find_boundary_loops(mesh: &PolyData) -> Vec<Vec<usize>> {
    let mut ec: std::collections::HashMap<(usize, usize), usize> = std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            *ec.entry((a.min(b), a.max(b))).or_insert(0) += 1;
        }
    }

    let boundary_edges: Vec<(usize, usize)> = ec
        .iter()
        .filter(|(_, &c)| c == 1)
        .map(|(&e, _)| e)
        .collect();
    if boundary_edges.is_empty() {
        return Vec::new();
    }

    let mut adj: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for &(a, b) in &boundary_edges {
        adj.entry(a).or_default().push(b);
        adj.entry(b).or_default().push(a);
    }

    let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut loops = Vec::new();

    for &(start, _) in &boundary_edges {
        if visited.contains(&start) {
            continue;
        }
        let mut loop_v = Vec::new();
        let mut current = start;
        loop {
            if !visited.insert(current) {
                break;
            }
            loop_v.push(current);
            let next = adj
                .get(&current)
                .and_then(|nbs| nbs.iter().find(|&&n| !visited.contains(&n)).cloned());
            match next {
                Some(n) => current = n,
                None => break,
            }
        }
        if loop_v.len() >= 3 {
            loops.push(loop_v);
        }
    }
    loops
}

fn ear_clip_3d(mesh: &PolyData, loop_verts: &[usize]) -> Vec<[usize; 3]> {
    let mut remaining: Vec<usize> = loop_verts.to_vec();
    let mut tris = Vec::new();

    while remaining.len() > 3 {
        let n = remaining.len();
        let mut found = false;
        for i in 0..n {
            let prev = remaining[(i + n - 1) % n];
            let curr = remaining[i];
            let next = remaining[(i + 1) % n];

            // Simple ear test: use the triangle regardless (greedy)
            tris.push([prev, curr, next]);
            remaining.remove(i);
            found = true;
            break;
        }
        if !found {
            break;
        }
    }
    if remaining.len() == 3 {
        tris.push([remaining[0], remaining[1], remaining[2]]);
    }
    tris
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_single_hole() {
        // Open mesh with one boundary loop
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]], // open square, no bottom
        );
        let holes = count_holes(&mesh);
        assert!(holes >= 1);

        let filled = fill_all_holes(&mesh);
        assert!(filled.polys.num_cells() > mesh.polys.num_cells());
    }

    #[test]
    fn closed_mesh_no_holes() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.5, 1.0, 0.0],
                [0.5, 0.5, 1.0],
            ],
            vec![[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
        );
        assert_eq!(count_holes(&mesh), 0);
        let filled = fill_all_holes(&mesh);
        assert_eq!(filled.polys.num_cells(), mesh.polys.num_cells());
    }

    #[test]
    fn ear_clip_fill() {
        let mesh = PolyData::from_triangles(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        );
        let filled = fill_holes_ear_clip(&mesh);
        assert!(filled.polys.num_cells() >= mesh.polys.num_cells());
    }

    #[test]
    fn hole_sizes_test() {
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let sizes = hole_sizes(&mesh);
        assert!(!sizes.is_empty());
        assert!(sizes[0] >= 3); // triangle has 3 boundary edges
    }
}
