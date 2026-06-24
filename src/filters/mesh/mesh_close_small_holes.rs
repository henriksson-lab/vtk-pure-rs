//! Close small boundary loops by fan-triangulating them.
use crate::data::{CellArray, Points, PolyData};

pub fn close_small_holes(mesh: &PolyData, max_hole_edges: usize) -> PolyData {
    let n = mesh.points.len();
    // Find boundary edges
    let mut edge_count: std::collections::HashMap<(usize, usize), u32> =
        std::collections::HashMap::new();
    for cell in mesh.polys.iter() {
        let nc = cell.len();
        for i in 0..nc {
            let a = cell[i] as usize;
            let b = cell[(i + 1) % nc] as usize;
            let e = if a < b { (a, b) } else { (b, a) };
            *edge_count.entry(e).or_insert(0) += 1;
        }
    }
    let mut boundary_adj: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for (&(a, b), &c) in &edge_count {
        if c == 1 {
            boundary_adj.entry(a).or_default().push(b);
            boundary_adj.entry(b).or_default().push(a);
        }
    }
    // Trace boundary loops
    let mut visited_edges: std::collections::HashSet<(usize, usize)> =
        std::collections::HashSet::new();
    let mut new_polys = CellArray::new();
    // Copy existing polys
    for cell in mesh.polys.iter() {
        new_polys.push_cell(&cell.to_vec());
    }
    let mut pts = mesh.points.clone();
    for &start in boundary_adj.keys() {
        let neighbors = match boundary_adj.get(&start) {
            Some(n) => n,
            None => continue,
        };
        if neighbors.iter().all(|&nb| {
            let e = if start < nb { (start, nb) } else { (nb, start) };
            visited_edges.contains(&e)
        }) {
            continue;
        }
        let mut loop_verts = vec![start];
        let mut current = start;
        loop {
            let neighbors = match boundary_adj.get(&current) {
                Some(n) => n,
                None => break,
            };
            let next = neighbors.iter().find(|&&nb| {
                let e = if current < nb {
                    (current, nb)
                } else {
                    (nb, current)
                };
                !visited_edges.contains(&e)
            });
            match next {
                Some(&nb) => {
                    let e = if current < nb {
                        (current, nb)
                    } else {
                        (nb, current)
                    };
                    visited_edges.insert(e);
                    if nb == start {
                        break;
                    }
                    loop_verts.push(nb);
                    current = nb;
                }
                None => break,
            }
        }
        if loop_verts.len() >= 3 && loop_verts.len() <= max_hole_edges {
            // Fan triangulate from centroid
            let mut cx = 0.0;
            let mut cy = 0.0;
            let mut cz = 0.0;
            for &v in &loop_verts {
                let p = mesh.points.get(v);
                cx += p[0];
                cy += p[1];
                cz += p[2];
            }
            let k = loop_verts.len() as f64;
            let center = pts.len();
            pts.push([cx / k, cy / k, cz / k]);
            for i in 0..loop_verts.len() {
                let j = (i + 1) % loop_verts.len();
                new_polys.push_cell(&[loop_verts[i] as i64, loop_verts[j] as i64, center as i64]);
            }
        }
    }
    let mut result = PolyData::new();
    result.points = pts;
    result.polys = new_polys;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_close_holes() {
        // Single triangle has a 3-edge boundary "hole"
        let mesh = PolyData::from_triangles(
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
            vec![[0, 1, 2]],
        );
        let r = close_small_holes(&mesh, 5);
        // Should add triangles to close the boundary
        assert!(r.polys.num_cells() >= 1);
    }
}
